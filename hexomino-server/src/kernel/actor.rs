use std::{future::Future, time::Duration};

use tokio::{
    select, spawn,
    sync::{
        mpsc::{self, Sender, UnboundedSender},
        oneshot::{self},
    },
    time::sleep,
};

pub trait Actor: Sized + Send + 'static {
    fn start(mut self) -> Addr<Self> {
        let (sender, mut receiver) = mpsc::channel(16);
        let (inner_sender, mut inner_receiver) = mpsc::unbounded_channel();
        let addr = Addr { sender };

        {
            let context = Context { inner_sender };
            self.started(&context);
            spawn(async move {
                let context = context;
                loop {
                    select! {
                        biased;
                        msg = inner_receiver.recv() => {
                            match msg.expect("inner_sender is dropped but shouldn't be") {
                                FastMsg::Stop => return,
                                FastMsg::Msg(msg) => msg.process_by(&mut self, &context),
                            }
                        }
                        msg = receiver.recv() => {
                            match msg {
                                Some(msg) => msg.process_by(&mut self, &context),
                                None => return
                            }
                        }
                    }
                }
            });
        }

        addr
    }

    fn started(&mut self, _ctx: &Context<Self>) {}
}

type SealedMsg<A> = Box<dyn ProcessBy<A> + Send>;

pub struct Addr<A: Actor> {
    sender: Sender<SealedMsg<A>>,
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
        }
    }
}

impl<A: Actor> Addr<A> {
    pub fn send<M>(&self, msg: M) -> impl Future<Output = anyhow::Result<<A as Handler<M>>::Output>>
    where
        A: Handler<M>,
        <A as Handler<M>>::Output: Send,
        M: Send + 'static,
    {
        let (result_sender, result_receiver) = oneshot::channel();
        let sender = self.sender.clone();
        async move {
            sender
                .send(Box::new(MsgReturnWrap {
                    msg,
                    respond_to: result_sender,
                }))
                .await
                .map_err(|_| anyhow::anyhow!("failed to send message to actor"))?;
            Ok(result_receiver.await?)
        }
    }

    #[allow(dead_code)]
    pub fn do_send<M>(&self, _msg: M)
    where
        A: Handler<M>,
        M: Send + 'static,
    {
        todo!();
    }
}

enum FastMsg<A: Actor> {
    Stop,
    Msg(SealedMsg<A>),
}

pub struct Context<A: Actor> {
    inner_sender: UnboundedSender<FastMsg<A>>,
}

impl<A: Actor> Clone for Context<A> {
    fn clone(&self) -> Self {
        Self {
            inner_sender: self.inner_sender.clone(),
        }
    }
}

impl<A: Actor> Context<A> {
    pub fn notify<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Send + 'static,
    {
        let _ = self
            .inner_sender
            .send(FastMsg::Msg(Box::new(MsgNoReturnWrap(msg))));
    }

    pub fn notify_later<M>(&self, msg: M, after: Duration)
    where
        A: Handler<M>,
        M: Send + 'static,
    {
        let inner_sender = self.inner_sender.clone();
        spawn(async move {
            sleep(after).await;
            let _ = inner_sender.send(FastMsg::Msg(Box::new(MsgNoReturnWrap(msg))));
        });
    }

    pub fn stop(&self) {
        let _ = self.inner_sender.send(FastMsg::Stop);
    }
}

pub trait Handler<M>: Actor {
    type Output;
    fn handle(&mut self, msg: M, ctx: &Context<Self>) -> Self::Output;
}

trait ProcessBy<A: Actor> {
    fn process_by(self: Box<Self>, actor: &mut A, ctx: &Context<A>);
}

struct MsgReturnWrap<M, OUT>
where
    M: Send,
    OUT: 'static,
{
    msg: M,
    respond_to: oneshot::Sender<OUT>,
}

impl<A, M> ProcessBy<A> for MsgReturnWrap<M, <A as Handler<M>>::Output>
where
    A: Handler<M>,
    M: Send,
{
    fn process_by(self: Box<Self>, actor: &mut A, ctx: &Context<A>) {
        let this = *self;
        let output = actor.handle(this.msg, ctx);
        let _ = this.respond_to.send(output);
    }
}

struct MsgNoReturnWrap<M>(M)
where
    M: Send;

impl<A, M> ProcessBy<A> for MsgNoReturnWrap<M>
where
    A: Handler<M>,
    M: Send,
{
    fn process_by(self: Box<Self>, actor: &mut A, ctx: &Context<A>) {
        let _ = actor.handle(self.0, ctx);
    }
}
