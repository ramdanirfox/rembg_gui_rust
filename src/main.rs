extern crate lazy_static;
extern crate native_windows_gui as nwg;
extern crate native_windows_derive as nwd;

use egui_backend::{
    egui,
    // fltk::{enums::*, prelude::*, *},
    fltk::{enums::EventState, enums::ColorDepth, enums::Cursor, enums::FrameType, enums::LabelType, prelude::*, *},
    gl, DpiScaling,
};
use fltk_egui as egui_backend;
use std::rc::Rc;
use std::{cell::RefCell, time::Instant};


use nfd::Response;
use nwd::NwgUi;
use nwg::NativeUi;
use std::{env};
use std::process::Command;
use redux_rs::{Store, Subscription};
use lazy_static::lazy_static;
use std::sync::Mutex;
use std::fs::File;
use std::io::Write;

const SCREEN_WIDTH: u32 = 800;
const SCREEN_HEIGHT: u32 = 600;

lazy_static! {
    static ref STORESAYA: Mutex<Store<State, Action>> = Mutex::new(Store::new(counter_reducer, State::default()));
    static ref PATH_FOLDER: Mutex<String> = Mutex::new("".to_string());
}
// const STORESAYA: Store<State, Action> = Store::new(counter_reducer, State::default());
//START OF REDUX
#[derive(Default)]
// This is a state. It describes an immutable object.
// It is changed via a 'reducer', a function which receives an action and returns a new state modified based on the action.
struct State {
    counter: i8,
    pathFolder: String
}

// The actions describe what the reducer has to do.
// Rust enums can carry a payload, which one can use to pass some value to the reducer.
enum Action {
    // Increment,
    // Decrement,
    SetPathFolder
}

// Here comes the reducer. It gets the current state plus an action to perform and returns a new state.
fn counter_reducer(state: &State, action: &Action) -> State {
    match action {
        // Action::Increment => State {
        //     counter: state.counter + 1,
        //     pathFolder: "".to_string()
        // },
        // Action::Decrement => State {
        //     counter: state.counter - 1,
        //     pathFolder: "".to_string()
        // },
        Action::SetPathFolder => State {
            counter: state.counter + 1,
            pathFolder: "".to_string()
        }
    }
}
//EOF REDUX

//START OF NWGUI
#[derive(Default, NwgUi)]
pub struct ImageDecoderApp {
    // The image that will be loaded dynamically
    loaded_image: RefCell<Option<nwg::Bitmap>>,

    #[nwg_control(size: (400, 300), position: (400, 150), title: "Image decoder")]
    #[nwg_events( OnWindowClose: [ImageDecoderApp::exit] )]
    window: nwg::Window,

    #[nwg_layout(parent: window, max_row: Some(5), max_column: Some(5) )]
    main_layout: nwg::GridLayout,

    #[nwg_resource]
    decoder: nwg::ImageDecoder,

    // #[nwg_resource(title: "Open File", action: nwg::FileDialogAction::Open, filters: "Png(*.png)|Jpeg(*.jpg;*.jpeg)|DDS(*.dds)|TIFF(*.tiff)|BMP(*.bmp)|Any (*.*)")]
    #[nwg_resource(title: "Open Polder", action: nwg::FileDialogAction::OpenDirectory)]
    dialog: nwg::FileDialog,
    
    

    #[nwg_control(text: "Open", focus: true)]
    #[nwg_layout_item(layout: main_layout, col: 0, row: 0)]
    // #[nwg_events(OnButtonClick: [ImageDecoderApp::open_file])]
    #[nwg_events(OnPaint: [ImageDecoderApp::open_file])]
    open_btn: nwg::Button,

    #[nwg_control(readonly: true)]
    #[nwg_layout_item(layout: main_layout, col: 1, row: 0, col_span: 4)]
    file_name: nwg::TextInput,

    #[nwg_control]
    #[nwg_layout_item(layout: main_layout, col: 0, row: 1, col_span: 5, row_span: 4)]
    img: nwg::ImageFrame,
}

impl ImageDecoderApp {

    fn open_file(&self) {
        if let Ok(d) = env::current_dir() {
            if let Some(d) = d.to_str() {
                self.dialog.set_default_folder(d).expect("Failed to set default folder.");
            }
        }
        
        if self.dialog.run(Some(&self.window)) {
            self.file_name.set_text("");
            if let Ok(directory) = self.dialog.get_selected_item() {
                let dir = directory.into_string().unwrap();
                self.file_name.set_text(&dir);
                self.read_file();
            }
        }
    }

    fn read_file(&self) {
        println!("{}", self.file_name.text());
        PATH_FOLDER.lock().unwrap().clear();
        PATH_FOLDER.lock().unwrap().push_str(&self.file_name.text()[..]);
        self.exit();
        let image = match self.decoder.from_filename(&self.file_name.text()) {
            Ok(img) => img,
            Err(_) => { println!("Could not read image!"); return; }
        };
        
        println!("Frame count: {}", image.frame_count());
        println!("Format: {:?}", image.container_format());

        let frame = match image.frame(0) {
            Ok(bmp) => bmp,
            Err(_) => { println!("Could not read image frame!"); return; }
        };

        println!("Resolution: {:?}", frame.resolution());
        println!("Size: {:?}", frame.size());

        // Create a new Bitmap image from the image data
        match frame.as_bitmap() {
            Ok(bitmap) => {
                let mut img = self.loaded_image.borrow_mut();
                img.replace(bitmap);
                self.img.set_bitmap(img.as_ref());
            },
            Err(_) => { println!("Could not convert image to bitmap!"); }
        }
    }

    fn exit(&self) {
        nwg::stop_thread_dispatch();
    }

}
//EOF NWGUI

fn main() {
    let mut incr: i64 = 1;
    let a = app::App::default().with_scheme(app::Scheme::Gtk);
    app::get_system_colors();
    app::set_font_size(20);
    let mut main_win = window::Window::new(100, 100, SCREEN_WIDTH as _, SCREEN_HEIGHT as _, None);
    let mut glut_win = window::GlWindow::new(5, 5, main_win.w() - 200, main_win.h() - 10, None);
    glut_win.set_mode(egui_backend::fltk::enums::Mode::Opengl3);
    glut_win.end();
    let mut col = group::Flex::default()
        .column()
        .with_size(185, 590)
        .right_of(&glut_win, 5);
    col.set_frame(FrameType::DownBox);
    let mut frm = frame::Frame::default();
    frm.set_color(egui_backend::fltk::enums::Color::Red.inactive());
    frm.set_frame(FrameType::FlatBox);

    let mut slider = valuator::Slider::default().with_type(valuator::SliderType::HorizontalFill);
    slider.set_slider_frame(FrameType::RFlatBox);
    slider.set_slider_size(0.20);
    slider.set_color(egui_backend::fltk::enums::Color::Blue.inactive());
    slider.set_selection_color(egui_backend::fltk::enums::Color::from_rgb(0, 231, 255));
    col.set_size(&mut slider, 20);
    col.end();
    main_win.end();
    main_win.make_resizable(true);
    main_win.show();
    glut_win.make_current();

    let (painter, egui_input_state) =
        egui_backend::with_fltk(&mut glut_win, DpiScaling::Custom(1.5));
    let mut egui_ctx = egui::CtxRef::default();

    let state_rc = Rc::from(RefCell::from(egui_input_state));
    let painter_rc = Rc::from(RefCell::from(painter));
    let state = state_rc.clone();
    let painter = painter_rc.clone();
    main_win.handle({
        let mut w = glut_win.clone();
        move |_, ev| match ev {
            enums::Event::Push
            | enums::Event::Released
            | enums::Event::KeyDown
            | enums::Event::KeyUp
            | enums::Event::MouseWheel
            | enums::Event::Resize
            | enums::Event::Move
            | enums::Event::Drag => {
                let mut state = state.borrow_mut();
                // println!("Cetak {} {}", ev.to_string(), incr);
                // incr += 1;
                state.fuse_input(&mut w, ev, &mut painter.borrow_mut());
                true
            }
            _ => false,
        }
    });

    let start_time = Instant::now();
    let mut name = String::new();
    let mut age = 0;
    let mut quit = false;
    let mut berkas_terpilih = std::string::String::from("Klik tombol di sebelah kiri yaa...");
    let mut output_cmd = std::string::String::from("disini");
    let mut output_cv = std::string::String::from("hasil");

    while a.wait() {
        let mut state = state_rc.borrow_mut();
        let mut painter = painter_rc.borrow_mut();
        state.input.time = Some(start_time.elapsed().as_secs_f64());
        egui_ctx.begin_frame(state.input.take());
        frm.set_label(&format!("Halo {}", &name));
        slider.set_value(age as f64 / 120.);

        unsafe {
            // Clear the screen to black
            gl::ClearColor(0.6, 0.3, 0.3, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }

        egui::CentralPanel::default().show(&egui_ctx, |ui| {
            ui.heading("Antarmuka Rembg");        
            ui.horizontal(|ui| {
                ui.label("Nama : ");
                ui.text_edit_singleline(&mut name);
            });
            ui.separator();
            ui.horizontal(|ui| {
                let berkasButton = ui.button("Pilih berkas...");
                // ui.style_mut().body_text_style = Color::from_rgb(255, 255, 255);
                    //   let mut berkasButtonStyle = berkasButton;
            // berkasButtonStyle.visuals.override_text_color = Some(egui::epaint::Color32::from_rgb(0, 0, 255));
            // berkasButton.ctx.set_style(berkasButtonStyle); 
            ui.visuals_mut().dark_mode = false;
            ui.visuals_mut().resize_corner_size = 80.;
                if berkasButton.clicked() {
                    println!("Memilih berkass...");
    
                    let result = nfd::dialog_multiple().filter("*").open().unwrap_or_else(|e| {
                        panic!(e);
                    });
    
                    match result {
                        Response::Okay(file_path) => println!("File path = {:?}", file_path),
                        Response::OkayMultiple(files) => { 
                            println!("Files {:?}", files);
                            berkas_terpilih = files[0].clone();
                            // Command::new("cmd")
                            //     .args(["/C", "echo hello"])
                            //     .output()
                            //     .expect("failed to execute process");
                            let output = Command::new("cmd")
                                .arg("/C")
                                .arg("dir /Q")
                                .output()
                                .expect("failed to execute process");
                            // println!("Dir result {:#?}", output);
                            println!("Dir result {:#?}", String::from_utf8_lossy(&output.stdout[..]));
                            // let out = output.stdout.to_string();
                            output_cmd = String::from_utf8_lossy(&output.stdout[..]).to_string();

                            // let outputcv = Command::new("cmd")
                            // .arg("/C")
                            // .arg(format!("type \"{}\" | progs\\app.exe > hasil.png", berkas_terpilih))
                            // .output()
                            // .expect("failed to execute process");
                            writeConverter(format!("cmd /C type \"{}\" | progs\\app.exe > hasil.png", berkas_terpilih));
                            let outputcv = Command::new("cmd")
                            .arg("/C")
                            .arg("konversi.bat")
                            .output()
                            .expect("failed to execute process");

                            output_cv = String::from_utf8_lossy(&outputcv.stdout[..]).to_string();
                        println!("Dir result {:#?}", output);
                        },
                        Response::Cancel => println!("User canceled"),
                    }
                }
                ui.label(&berkas_terpilih);
            });
            ui.separator();
            ui.label(&output_cmd);
            ui.separator();
            ui.label(&output_cv);
            ui.separator();
            ui.horizontal(|ui| {
                if ui.button("Buka folder...").clicked() {
                
                    println!("keklik ngab! {}", PATH_FOLDER.lock().unwrap());
                    nwg::init().expect("Failed to init Native Windows GUI");
                    nwg::Font::set_global_family("Segoe UI").expect("Failed to set default font");
                
                    let _app = ImageDecoderApp::build_ui(Default::default()).expect("Failed to build UI");
                
                    nwg::dispatch_thread_events();
                }

                if ui.button("Buka Folder Output...").clicked() {
                    writeConverter(format!("explorer %cd%"));
                    let outputexplorer = Command::new("cmd")
                    .arg("/C")
                    .arg("konversi.bat")
                    .output()
                    .expect("failed to execute process");
                }
            });
            ui.separator();
            ui.add(egui::Slider::new(&mut age, 0..=120).text("age"));
            if ui.button("Click each yearzzz").clicked() {
                age += 1;
                println!("Kepencet gays");

            }
            ui.label(format!("Hello '{}', age {}", name, age));
            ui.separator();
            if ui
                .button("Quit?")
                .on_hover_cursor(egui::CursorIcon::PointingHand)
                .clicked()
            {
                quit = true;
            }
        });

        let (egui_output, paint_cmds) = egui_ctx.end_frame();
        state.fuse_output(&mut glut_win, &egui_output);

        let paint_jobs = egui_ctx.tessellate(paint_cmds);

        //Draw egui texture
        painter.paint_jobs(None, paint_jobs, &egui_ctx.texture());

        glut_win.swap_buffers();
        glut_win.flush();
        app::sleep(0.006);
        app::awake();
        if quit {
            break;
        }
    }
}

fn writeConverter(text: String) {
    let data = "Some data!";
    let mut f = File::create("konversi.bat").expect("Unable to create file");
    f.write_all(text.as_bytes()).expect("Unable to write data");
}