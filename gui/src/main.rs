use std::{env, ops::Index, path::PathBuf, process::Command};

use druid::{
    AppDelegate, AppLauncher, Color, Data, Env, EventCtx, Lens, LocalizedString, PlatformError,
    Selector, Widget, WidgetExt, WindowDesc,
    platform_menus::win::file::exit,
    widget::{Align, Button, Container, Either, Flex, Label, List, Padding, Scope, Scroll},
};
use rfd::FileDialog;
use std::sync::Arc;

#[derive(Clone, Data, Lens)]
struct AppState {
    selected_folders: Arc<Vec<String>>,
    page: Page,
}

#[derive(Clone, Data, PartialEq)]
enum Page {
    PreOp,
    PostOp,
}

fn get_exe_dir() -> PathBuf {
    env::current_exe().unwrap().parent().unwrap().to_path_buf()
}

// const WINDOW_WIDTH: f64 = 500.0;
// const WINDOW_HEIGHT: f64 = 600.0;

fn main() -> Result<(), PlatformError> {
    println!("{:?}", get_exe_dir().join("file-duplicate-finder"));
    let main_window = WindowDesc::new(ui_builder())
        /* .with_min_size((WINDOW_WIDTH, WINDOW_HEIGHT)) */;
    let initial_state = AppState {
        selected_folders: Arc::new(Vec::new()),
        page: Page::PreOp,
    };
    AppLauncher::with_window(main_window)
        .delegate(Delegate)
        .log_to_console()
        .launch(initial_state)
}

fn ui_builder() -> impl Widget<AppState> {
    Either::new(
        |data: &AppState, _env| data.page == Page::PreOp,
        pre_op_page(),
        post_op_page(),
    )
}

const FOLDER_CLICKED: Selector<String> = Selector::new("app.folder-clicked");

fn pre_op_page() -> impl Widget<AppState> {
    let folder_select_button =
        Button::new("Select Folders").on_click(|_ctx, data: &mut AppState, _env| {
            if let Some(folders) = FileDialog::new()
                .set_directory(env::current_dir().unwrap())
                .pick_folders()
            {
                let folders = folders
                    .into_iter()
                    .map(|f| f.display().to_string())
                    .collect::<Vec<String>>();

                let result = data
                    .selected_folders
                    .iter()
                    .cloned()
                    .chain(folders.into_iter())
                    .collect::<Vec<String>>();

                data.selected_folders = Arc::new(result);
            }
        });

    let folder_list = Scroll::new(
        List::new(|| {
            Button::new(|item: &String, _env: &Env| item.clone()).on_click(
                |ctx, item: &mut String, _env| {
                    ctx.submit_command(FOLDER_CLICKED.with(item.clone()));
                    ctx.request_update();
                },
            )
        })
        .lens(AppState::selected_folders),
    );
    let folder_container = Container::new(folder_list)
        .border(Color::WHITE, 2.0)
        .rounded(10.0)
        .expand_width()
        .padding(5.0);

    let run_button =
        Button::new("Run scan").on_click(|ctx: &mut EventCtx, data: &mut AppState, _env| {
            for folder in data.selected_folders.iter() {
                println!("{:?}", folder);
            }
            let output = Command::new(get_exe_dir().join("file-duplicate-finder"))
                .args(data.selected_folders.iter())
                .output();

            if let Ok(out) = output {
                println!(
                    "{:?}\n{}\n{}",
                    out.status.code(),
                    String::from_utf8_lossy(&out.stdout),
                    String::from_utf8_lossy(&out.stdout)
                );

                if out.status.code().is_none_or(|x| x != 0) {
                    let error_window = WindowDesc::new(error_popup())
                        .title("Error")
                        .window_size((300., 150.));
                    ctx.new_window(error_window);
                } else {
                    data.page = Page::PostOp;
                    ctx.request_update();
                }
            }
        });

    Flex::column()
        .with_flex_spacer(0.5)
        .with_child(folder_select_button)
        .with_child(Label::new("Selected Folders:"))
        .with_child(folder_container)
        .with_child(run_button)
        .with_flex_spacer(1.0)
}

fn error_popup() -> impl Widget<AppState> {
    Flex::column()
        .with_child(
            Label::new("An error occured while trying to scan the directory")
                .with_line_break_mode(druid::widget::LineBreaking::WordWrap),
        )
        .with_child(Button::new("Close").on_click(|ctx, _, _| {
            ctx.window().close();
        }))
}

fn post_op_page() -> impl Widget<AppState> {
    Align::centered(
        Flex::column()
            .with_flex_spacer(1.0)
            .with_child(Label::new("Scan complete").padding(10.))
            .with_child(
                Button::new("View scan log")
                    .on_click(|_, _, _| {
                        let _ = open::that(get_exe_dir().join("results.log"));
                    })
                    .padding(10.),
            )
            .with_flex_child(
                Flex::row()
                    .with_child(
                        Button::new("Go back")
                            .on_click(|ctx: &mut EventCtx, data: &mut AppState, _| {
                                data.page = Page::PreOp;
                                ctx.request_update();
                            })
                            .padding(10.),
                    )
                    .with_child(
                        Button::new("Exit")
                            .on_click(|ctx, _, _| ctx.window().close())
                            .padding(10.),
                    ),
                1.0,
            )
            .with_flex_spacer(1.0),
    )
}

struct Delegate;

impl AppDelegate<AppState> for Delegate {
    fn command(
        &mut self,
        _ctx: &mut druid::DelegateCtx,
        _target: druid::Target,
        cmd: &druid::Command,
        data: &mut AppState,
        _env: &Env,
    ) -> druid::Handled {
        if let Some(folder) = cmd.get(FOLDER_CLICKED) {
            data.selected_folders = Arc::new(
                data.selected_folders
                    .iter()
                    .filter(|x| *x != folder)
                    .cloned()
                    .collect(),
            );

            return druid::Handled::Yes;
        }
        druid::Handled::No
    }
}
