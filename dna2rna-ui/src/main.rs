use fltk::{app::*, dialog::*, menu::*, window::Window};
use std::{fs, path};

#[derive(Copy, Clone)]
pub enum Message {
    Open,
    Quit,
    About,
}

fn main() {
    let app = App::default().with_scheme(AppScheme::Gtk);

    let (s, r) = channel::<Message>();

    let mut wind = Window::default()
        .with_size(640, 700)
        .center_screen()
        .with_label("DNA 2 RNA");

    let mut menu = MenuBar::new(0, 0, 640, 40, "");
    menu.set_color(Color::Light2);

    menu.add_emit(
        "File/Open...",
        Shortcut::Ctrl + 'o',
        MenuFlag::Normal,
        s,
        Message::Open,
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

    while app.wait().expect("Couldn't run editor!") {
        use Message::*;
        match r.recv() {
            Some(msg) => match msg {
                Open => {
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
                            let dna = fs::read_to_string(filename).unwrap();
                            let f = dna2rna::Dna2Rna::new(&dna);
                            d2r = Some(f);
                        }
                        false => alert(200, 200, "File does not exist!"),
                    }
                }
                Quit => app.quit(),
                About => message(200, 200, "DNA 2 RNA"),
            },
            _ => (),
        }
    }
}
