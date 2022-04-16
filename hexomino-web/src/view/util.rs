use web_sys::HtmlCanvasElement;

pub fn resize_canvas_and_return_size(canvas: &HtmlCanvasElement) -> (f64, f64) {
    let width = canvas.client_width() as u32;
    let height = canvas.client_height() as u32;
    canvas.set_width(width);
    canvas.set_height(height);
    (width as f64, height as f64)
}
