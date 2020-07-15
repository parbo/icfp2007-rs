use fltk::{app::*, dialog::*, draw::*, frame::*, menu::*, window::Window};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::{fs, path};

#[derive(Copy, Clone)]
pub enum Message {
    StepDNA,
    OpenDNA,
    StepRNA(bool),
    OpenRNA,
    Quit,
    About,
    Test,
}

struct EndoRnaStore<F>
where
    F: Fn(String) -> (),
{
    pub cls: Box<F>,
}

impl<F> dna2rna::RnaStore for EndoRnaStore<F>
where
    F: Fn(String) -> (),
{
    fn store(&mut self, rna: String) {
        (*self.cls)(rna);
    }
}

struct Endo<'a> {
    pub d2r: dna2rna::Dna2Rna<'a>,
    pub fuun: rna2fuun::Fuun,
    pub step_dna: usize,
    pub steps: usize,
}

impl<'a> Endo<'a> {
    pub fn new(rna_store: &'a mut dyn dna2rna::RnaStore) -> Endo<'a> {
        Endo {
            d2r: dna2rna::Dna2Rna::new(rna_store),
            fuun: rna2fuun::Fuun::new(""),
            step_dna: 5000,
            steps: 0,
        }
    }
}

fn main() {
    env_logger::init();

    let app = App::default().with_scheme(AppScheme::Gtk);

    let (tx, rx): (Sender<[char; 7]>, Receiver<[char; 7]>) = mpsc::channel();
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

    let mut rna_store = EndoRnaStore {
        cls: Box::new(move |rna: String| {
            assert_eq!(rna.len(), 7);
            let mut chars = rna.chars();
            let c = [
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
                chars.next().unwrap(),
            ];
            tx.send(c).expect("error");
        }),
    };
    let mut endo = Endo::new(&mut rna_store);

    s.send(Message::Test);

    while app.wait().expect("Couldn't run editor!") {
        use Message::*;
        match r.recv() {
            Some(msg) => match msg {
                StepRNA(dna) => {
                    log::info!("rna..");
                    while let Ok(c) = rx.try_recv() {
                        let mut s = String::new();
                        s.push(c[0]);
                        s.push(c[1]);
                        s.push(c[2]);
                        s.push(c[3]);
                        s.push(c[4]);
                        s.push(c[5]);
                        s.push(c[6]);
                        endo.fuun.add_rna_command(s);
                    }
                    let rem = endo.fuun.remaining_steps();
                    let (bmp, done) = endo.fuun.step(rem);
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
                        s.send(Message::StepRNA(dna));
                    } else if dna {
                        s.send(Message::StepDNA);
                    }
                    log::info!("..rna");
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
                            endo.fuun.reset();
                            endo.fuun.add_rna_str(&rna);
                            s.send(Message::StepRNA(false));
                        }
                        false => alert(200, 200, "File does not exist!"),
                    }
                }
                StepDNA => {
                    log::info!("dna.. {}", endo.steps);
                    let mut done = false;
                    for _ in 0..endo.step_dna {
                        endo.steps = endo.steps + 1;
                        if endo.d2r.execute_step() {
                            done = true;
                            break;
                        }
                    }
                    s.send(Message::StepRNA(!done));
                    log::info!("..dna");
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
                            //let prefix = Some("IIPIFFCPICICIICPIICIPPPICIIC");
                            let prefix = None;
                            endo.d2r.set_dna_and_prefix(&dna, prefix);
                            endo.fuun.reset();
                            endo.steps = 0;
                            s.send(Message::StepDNA);
                        }
                        false => alert(200, 200, "File does not exist!"),
                    }
                }
                Quit => app.quit(),
                About => message(200, 200, "Endo"),
                Test => {
                    log::info!("testing...");
                }
            },
            _ => (),
        }
    }
}
