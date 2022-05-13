use parking_lot::Mutex;
use std::{collections::VecDeque, future::Future, sync::Arc};
use tokio::{
    spawn,
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
};

pub trait Actor: Sized + Send + 'static {
    fn start(mut self) -> Addr<Self> {
        let (sender, mut receiver) = mpsc::channel(16);
        let fast_queue = Arc::new(Mutex::new(VecDeque::new()));
        let addr = Addr {
            sender,
            fast_queue: fast_queue.clone(),
        };

        {
            let context = Context { fast_queue };
            spawn(async move {
                while let Some(msg) = receiver.recv().await {
                    msg.process_by(&mut self, &context);
                }
            });
        }

        addr
    }

    //fn started(&mut self
}

type SealedMsg<A> = Box<dyn ProcessBy<A> + Send>;

pub struct Addr<A: Actor> {
    sender: Sender<SealedMsg<A>>,
    fast_queue: Arc<Mutex<VecDeque<FastMsg<A>>>>,
}

impl<A: Actor> Clone for Addr<A> {
    fn clone(&self) -> Self {
        Self {
            sender: self.sender.clone(),
            fast_queue: self.fast_queue.clone(),
        }
    }
}

impl<A: Actor> Addr<A> {
    pub fn send<M>(&self, msg: M) -> impl Future<Output = <A as Handler<M>>::Output>
    where
        A: Handler<M>,
        <A as Handler<M>>::Output: Send,
        M: Send + 'static,
    {
        let (result_sender, result_receiver) = oneshot::channel();
        let sender = self.sender.clone();
        async move {
            let _ = sender
                .send(Box::new(MsgReturnWrap {
                    msg,
                    respond_to: result_sender,
                }))
                .await
                .map_err(|_| ());
            result_receiver.await.expect("receive result failed")
        }
    }

    #[allow(dead_code)]
    pub fn do_send<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Send + 'static,
    {
        self.fast_queue
            .lock()
            .push_back(FastMsg::Msg(Box::new(MsgNoReturnWrap(msg))));
    }
}

#[allow(dead_code)]
enum FastMsg<A: Actor> {
    Stop,
    Msg(SealedMsg<A>),
}

pub struct Context<A: Actor> {
    fast_queue: Arc<Mutex<VecDeque<FastMsg<A>>>>,
}

impl<A: Actor> Clone for Context<A> {
    fn clone(&self) -> Self {
        Self {
            fast_queue: self.fast_queue.clone(),
        }
    }
}

#[allow(dead_code)]
impl<A: Actor> Context<A> {
    pub fn notify<M>(&self, msg: M)
    where
        A: Handler<M>,
        M: Send + 'static,
    {
        self.fast_queue
            .lock()
            .push_back(FastMsg::Msg(Box::new(MsgNoReturnWrap(msg))));
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
