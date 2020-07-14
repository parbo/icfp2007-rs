use fltk::{app::*, dialog::*, draw::*, frame::*, menu::*, window::Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::{fs, path};

#[derive(Copy, Clone)]
pub enum Message {
    StepDNA,
    OpenDNA,
    StepRNA,
    OpenRNA,
    Quit,
    About,
}

fn main() {
    let app = App::default().with_scheme(AppScheme::Gtk);

    let (s, r) = channel::<Message>();

    let mut wind = Window::default()
        .with_size(640, 700)
        .center_screen()
        .with_label("Endo");

    let mut menu = MenuBar::new(0, 0, 640, 40, "");
    menu.set_color(Color::Light2);

    let mut frame = Frame::new(5, 40, 600, 600, "");
    frame.set_color(Color::White);
    let frame_c = frame.clone();

    let offscreen = Offscreen::new(600, 600).unwrap();
    offscreen.begin();
    set_draw_color(Color::White);
    draw_rectf(0, 0, 600, 600);
    offscreen.end();

    let offs = Rc::from(RefCell::from(offscreen));
    let offs_rc = offs.clone();

    frame.draw(Box::new(move || {
        if offs_rc.borrow().is_valid() {
            offs_rc
                .borrow()
                .copy(frame_c.x(), frame_c.y(), 600, 600, 0, 0);
        }
    }));

    menu.add_emit(
        "File/Open DNA...",
        Shortcut::Ctrl + 'o',
        MenuFlag::Normal,
        s,
        Message::OpenDNA,
    );

    menu.add_emit(
        "File/Open RNA...",
        Shortcut::Ctrl + 'r',
        MenuFlag::Normal,
        s,
        Message::OpenRNA,
    );

    menu.add_emit(
        "File/Quit",
        Shortcut::None,
        MenuFlag::Normal,
        s,
        Message::Quit,
    );

    menu.add_emit(
        "Help/About",
        Shortcut::None,
        MenuFlag::Normal,
        s,
        Message::About,
    );

    let mut x = menu.find_item("File/Quit").unwrap();
    x.set_label_color(Color::Red);

    wind.make_resizable(false);
    wind.end();
    wind.show();

    wind.set_callback(Box::new(move || {
        if event() == Event::Close {
            s.send(Message::Quit);
        }
    }));

    let mut d2r: Option<dna2rna::Dna2Rna> = None;
    let mut fuun: Option<rna2fuun::Fuun> = None;
    let step_dna = 1000;
    let mut step_rna = 0;

    while app.wait().expect("Couldn't run editor!") {
        use Message::*;
        match r.recv() {
            Some(msg) => match msg {
                StepRNA => {
                    if let Some(f) = &mut fuun {
                        let (bmp, done) = f.step(step_rna);
                        offs.borrow().begin();
                        for y in 0..600 {
                            for x in 0..600 {
                                let ix = (y * 600 + x) as usize;
                                let pixel = bmp.pixels[ix];
                                set_color_rgb(pixel.color.r, pixel.color.g, pixel.color.b);
                                draw_point(x, y);
                            }
                        }
                        offs.borrow().end();
                        frame.redraw();
                        if !done {
                            s.send(Message::StepRNA);
                        } else {
                            s.send(Message::StepDNA);
			}
                    }
                }
                OpenRNA => {
                    let mut dlg = FileDialog::new(FileDialogType::BrowseFile);
                    dlg.set_option(FileDialogOptions::NoOptions);
                    dlg.set_filter("*.rna");
                    dlg.show();
                    let filename = dlg.filename().to_string_lossy().to_string();
                    if filename.is_empty() {
                        return;
                    }
                    match path::Path::new(&filename).exists() {
                        true => {
                            let rna = fs::read_to_string(filename).unwrap();
                            step_rna = rna.len() / 7;
                            let f = rna2fuun::Fuun::new(&rna);
                            fuun = Some(f);
                            s.send(Message::StepRNA);
                        }
                        false => alert(200, 200, "File does not exist!"),
                    }
                }
                StepDNA => {
                    if let Some(d) = &mut d2r {
			let mut done = false;
			for _ in 0..step_dna {
                            if d.execute_step() {
				done = true;
				break;
			    }
			}
                        if !done {
			    let rna = d.rna.to_string();
                            let f = rna2fuun::Fuun::new(&rna);
                            step_rna = rna.len();
                            fuun = Some(f);
                            s.send(Message::StepRNA);
                        }
                    }
                }
                OpenDNA => {
                    let mut dlg = FileDialog::new(FileDialogType::BrowseFile);
                    dlg.set_option(FileDialogOptions::NoOptions);
                    dlg.set_filter("*.dna");
                    dlg.show();
                    let filename = dlg.filename().to_string_lossy().to_string();
                    if filename.is_empty() {
                        return;
                    }
                    match path::Path::new(&filename).exists() {
                        true => {
                            let dna = fs::read_to_string(filename).unwrap();
			    // TODO: prefixes
                            let d = dna2rna::Dna2Rna::new(&dna, Some("IIPIFFCPICICIICPIICIPPPICIIC"));
                            d2r = Some(d);
                            s.send(Message::StepDNA);
                        }
                        false => alert(200, 200, "File does not exist!"),
                    }
                }
                Quit => app.quit(),
                About => message(200, 200, "Endo"),
            },
            _ => (),
        }
    }
}
