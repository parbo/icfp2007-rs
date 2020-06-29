use fltk::{app::*, button::*, frame::*, window::*};
use rna2fuun;

fn main() {
    let app = App::default();
    let mut wind = Window::new(100, 100, 400, 300, "Hello from rust");
    let mut frame = Frame::new(0, 0, 400, 200, "");
    let mut but = Button::new(160, 210, 80, 40, "Click me!");
    let mut ctr = 0;
    wind.end();
    wind.show();
    but.set_callback(Box::new(move || {
	ctr = rna2fuun::add_one(ctr);
	let s = format!("{}", ctr);
	frame.set_label(&s);
    }
    ));
    app.run().unwrap();
}
