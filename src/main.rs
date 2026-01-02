use eframe::egui;
use local_issues_lib::{
    Comment, Issue, Issues, Status,
    user::{User, Users},
};
use std::collections::HashMap;
use std::fs;

// ----------------------------------------------------------------------------
// 1. define filtering
// ----------------------------------------------------------------------------
#[derive(PartialEq)]
enum FilterStatus {
    Open,
    Completed,  // "CloseAsCmp"
    NotPlanned, // "CloseAsNotPlaned"
    All,
}

impl FilterStatus {
    fn matches(&self, status: &Status) -> bool {
        match (self, status) {
            (FilterStatus::Open, Status::Open) => true,
            (FilterStatus::Open, _) => false,
            (FilterStatus::Completed, Status::CloseAsCmp) => true,
            (FilterStatus::Completed, Status::CloseAsForked) => true, // ForkedもCompleted扱いとする
            (FilterStatus::Completed, _) => false,
            (FilterStatus::NotPlanned, Status::CloseAsNotPlaned) => true,
            (FilterStatus::NotPlanned, _) => false,
            (FilterStatus::All, _) => true,
        }
    }
}

// ----------------------------------------------------------------------------
// 2. アプリケーション構造体
// ----------------------------------------------------------------------------
struct RamifiApp {
    issues: Issues,
    users: Users,

    // UI State
    new_description: String,

    // User Manager UI State
    show_user_manager: bool,
    new_user_name: String,
    new_user_email: String,

    // Navigation / Action State
    comment_drafts: HashMap<usize, String>,
    filter_status: FilterStatus,
    query: String,

    // 選択中のIssue ID
    selected_issue_index: Option<usize>,

    current_user: User,
}

impl RamifiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Self::setup_custom_fonts(&cc.egui_ctx);

        let mut users = Users::new();
        let default_user = User::new("coyuki", "coyuki@example.com");
        users.add_user(default_user.clone());
        let current_user = default_user;

        let mut issues = Issues::new();

        // --- 初期データ ---
        let mut issue1 = Issue::new("GUI実装", current_user.clone(), vec!["Enhancement"]);
        issue1.comment(Comment::new(
            "eframeを使ってGUIを作る",
            current_user.clone(),
        ));
        issues.add_new_issue(issue1);

        let mut issue2 = Issue::new("永続化", current_user.clone(), vec!["Feature"]);
        issue2.comment(Comment::new("データを保存する", current_user.clone()));
        issues.add_new_issue(issue2);

        let mut issue3 = Issue::new("UI改善", current_user.clone(), vec!["Design"]);
        issue3.comment(Comment::new("見た目を良くする", current_user.clone()));
        issues.add_new_issue(issue3);

        Self {
            issues,
            users,
            new_description: String::new(),
            show_user_manager: false,
            new_user_name: String::new(),
            new_user_email: String::new(),
            comment_drafts: HashMap::new(),
            filter_status: FilterStatus::Open,
            query: String::new(),
            selected_issue_index: Some(0),
            current_user,
        }
    }

    fn setup_custom_fonts(ctx: &egui::Context) {
        use eframe::egui::{FontData, FontDefinitions, FontFamily};
        let mut fonts = FontDefinitions::default();
        let font_candidates = [
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/ipafont-gothic/ipag.ttf",
            "/usr/share/fonts/TTF/ipag.ttf",
            "C:\\Windows\\Fonts\\meiryo.ttc",
            "C:\\Windows\\Fonts\\msgothic.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
        ];

        for path in font_candidates {
            if let Ok(font_binary) = fs::read(path) {
                println!("Loaded font: {}", path);
                fonts.font_data.insert(
                    "my_system_font".to_owned(),
                    FontData::from_owned(font_binary).into(),
                );
                fonts
                    .families
                    .entry(FontFamily::Proportional)
                    .or_default()
                    .insert(0, "my_system_font".to_owned());
                fonts
                    .families
                    .entry(FontFamily::Monospace)
                    .or_default()
                    .insert(0, "my_system_font".to_owned());
                ctx.set_fonts(fonts);
                return;
            }
        }
        eprintln!("Japanese font not found in standard paths.");
    }
}

impl eframe::App for RamifiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // --- User Manager Window ---
        if self.show_user_manager {
            egui::Window::new("User Manager")
                .open(&mut self.show_user_manager)
                .show(ctx, |ui| {
                    ui.heading("Switch User");
                    let user_list = self.users.get_list().clone();
                    for user in user_list {
                        ui.horizontal(|ui| {
                            ui.label(format!("User: {}", user.name()));
                            if ui.button("Switch").clicked() {
                                self.current_user = user.clone();
                            }
                            if user.name() == self.current_user.name() {
                                ui.label("(Current)");
                            }
                        });
                    }
                    ui.separator();
                    ui.heading("Add New User");
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.new_user_name);
                    });
                    ui.horizontal(|ui| {
                        ui.label("Email:");
                        ui.text_edit_singleline(&mut self.new_user_email);
                    });
                    if ui.button("Create").clicked()
                        && !self.new_user_name.is_empty()
                        && !self.new_user_email.is_empty()
                    {
                        let new_u = User::new(&self.new_user_name, &self.new_user_email);
                        self.users.add_user(new_u);
                        self.new_user_name.clear();
                        self.new_user_email.clear();
                    }
                });
        }

        // --- Top Panel ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(
                    egui::RichText::new("Ramifi Issue Tracker")
                        .strong()
                        .size(16.0),
                );
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Manage Users").clicked() {
                        self.show_user_manager = true;
                    }
                    ui.label(format!("User: {}", self.current_user.name()));
                });
            });
        });

        // --- Left Side Panel (Issue List) ---
        egui::SidePanel::left("issue_list_panel")
            .resizable(true)
            .default_width(250.0)
            .show(ctx, |ui| {
                ui.add_space(5.0);

                // New Issue Input
                ui.horizontal(|ui| {
                    ui.text_edit_singleline(&mut self.new_description);
                    if ui.button("New").clicked() && !self.new_description.is_empty() {
                        let mut issue = Issue::new(
                            &self.new_description,
                            self.current_user.clone(),
                            vec![] as Vec<String>,
                        );
                        issue.comment(Comment::new(
                            self.new_description.clone(),
                            self.current_user.clone(),
                        ));
                        let new_index = self.issues.add_new_issue(issue);
                        self.new_description.clear();
                        self.selected_issue_index = Some(new_index);
                    }
                });

                ui.separator();

                // Filters (Enhanced)
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.filter_status, FilterStatus::Open, "Open");
                    ui.selectable_value(
                        &mut self.filter_status,
                        FilterStatus::Completed,
                        "Completed",
                    );
                    ui.selectable_value(
                        &mut self.filter_status,
                        FilterStatus::NotPlanned,
                        "Not Planned",
                    );
                    ui.selectable_value(&mut self.filter_status, FilterStatus::All, "All");
                });

                // Search UI (Enhanced)
                ui.horizontal(|ui| {
                    ui.label("Search:"); // Add Title
                    ui.text_edit_singleline(&mut self.query);
                });

                ui.separator();

                // Issue List
                egui::ScrollArea::vertical().show(ui, |ui| {
                    let all_issues = self.issues.get_list();
                    let mut display_issues: Vec<(usize, Issue)> = all_issues
                        .iter()
                        .enumerate()
                        .map(|(i, issue)| (i, issue.clone()))
                        .collect();

                    display_issues.retain(|(_, issue)| self.filter_status.matches(issue.status()));
                    if !self.query.is_empty() {
                        let query = self.query.to_lowercase();
                        display_issues
                            .retain(|(_, issue)| issue.name().to_lowercase().contains(&query));
                    }
                    display_issues.sort_by(|a, b| b.0.cmp(&a.0));

                    for (id, issue) in display_issues {
                        let is_selected = self.selected_issue_index == Some(id);
                        let (icon, _color) = match issue.status() {
                            Status::Open => ("🟢", egui::Color32::GREEN),
                            Status::CloseAsCmp => ("🔴", egui::Color32::RED),
                            Status::CloseAsNotPlaned => ("⚪", egui::Color32::GRAY),
                            Status::CloseAsForked => ("🔵", egui::Color32::BLUE),
                        };
                        let label = format!("{} #{} {}", icon, id + 1, issue.name());

                        if ui.selectable_label(is_selected, label).clicked() {
                            self.selected_issue_index = Some(id);
                        }
                    }
                });
            });

        // --- Central Panel (Issue Detail) ---
        egui::CentralPanel::default().show(ctx, |ui| {
            if let Some(id) = self.selected_issue_index {
                if let Some(issue) = self.issues.get(id).cloned() {
                    // --- Header Area ---
                    ui.horizontal(|ui| {
                        ui.heading(format!("{} #{}", issue.name(), id + 1));

                        let (status_text, status_bg) = match issue.status() {
                            Status::Open => (" Open ", egui::Color32::from_rgb(46, 160, 67)),
                            Status::CloseAsCmp => {
                                (" Completed ", egui::Color32::from_rgb(130, 80, 223))
                            }
                            Status::CloseAsNotPlaned => (" Not Planned ", egui::Color32::GRAY),
                            Status::CloseAsForked => (" Forked ", egui::Color32::BLUE),
                        };

                        ui.add(egui::Label::new(
                            egui::RichText::new(status_text)
                                .color(egui::Color32::WHITE)
                                .background_color(status_bg)
                                .strong(),
                        ));

                        if issue.from_index() != 0 {
                            let parent_display_id = issue.from_index() + 1;
                            if ui
                                .link(format!("Forked from #{}", parent_display_id))
                                .clicked()
                            {
                                self.filter_status = FilterStatus::All;
                                self.selected_issue_index = Some(issue.from_index());
                            }
                        }

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.label(
                                egui::RichText::new(
                                    issue.created_at().format("%Y-%m-%d %H:%M").to_string(),
                                )
                                .weak(),
                            );

                            ui.add_space(10.0);

                            // Labels
                            egui::ScrollArea::horizontal()
                                .id_source("header_labels_scroll")
                                .max_width(400.0)
                                .show(ui, |ui| {
                                    ui.with_layout(
                                        egui::Layout::left_to_right(egui::Align::Center),
                                        |ui| {
                                            for label in issue.get_labels() {
                                                ui.add(egui::Label::new(
                                                    egui::RichText::new(label)
                                                        .color(egui::Color32::BLACK)
                                                        .background_color(
                                                            egui::Color32::LIGHT_GRAY,
                                                        ),
                                                ));
                                            }
                                        },
                                    );
                                });
                        });
                    });

                    ui.separator();

                    // --- Main Content (Single Column) ---
                    ui.allocate_ui_with_layout(
                        egui::vec2(ui.available_width(), ui.available_height()),
                        egui::Layout::top_down(egui::Align::Min),
                        |ui| {
                            egui::ScrollArea::vertical()
                                .id_source("main_scroll")
                                .show(ui, |ui| {
                                    // Comments
                                    for comment in issue.comments().iter() {
                                        egui::Frame::group(ui.style()).inner_margin(8.0).show(
                                            ui,
                                            |ui| {
                                                ui.horizontal(|ui| {
                                                    ui.label(
                                                        egui::RichText::new(
                                                            comment.author().name(),
                                                        )
                                                        .strong(),
                                                    );
                                                    ui.label(
                                                        egui::RichText::new(
                                                            comment
                                                                .date()
                                                                .format("%Y-%m-%d %H:%M")
                                                                .to_string(),
                                                        )
                                                        .weak()
                                                        .size(10.0),
                                                    );
                                                });
                                                ui.separator();
                                                ui.label(comment.text());
                                            },
                                        );
                                        ui.add_space(8.0);
                                    }

                                    ui.add_space(10.0);
                                    ui.separator();

                                    ui.label(egui::RichText::new("Add a comment").strong());

                                    let draft_text = self.comment_drafts.entry(id).or_default();
                                    ui.add(
                                        egui::TextEdit::multiline(draft_text)
                                            .desired_width(f32::INFINITY)
                                            .hint_text("Leave a comment"),
                                    );

                                    ui.add_space(5.0);

                                    // Buttons
                                    ui.with_layout(
                                        egui::Layout::right_to_left(egui::Align::Min),
                                        |ui| {
                                            if ui.button("Comment").clicked()
                                                && !draft_text.is_empty()
                                                && let Some(target_issue) = self.issues.get_mut(id)
                                            {
                                                target_issue.comment(Comment::new(
                                                    draft_text.clone(),
                                                    self.current_user.clone(),
                                                ));
                                                draft_text.clear();
                                            }

                                            ui.add_space(5.0);

                                            // Actions / Close Menu
                                            let menu_label = if issue.status() == &Status::Open {
                                                "Close Issue ▾"
                                            } else {
                                                "Actions ▾"
                                            };

                                            ui.menu_button(menu_label, |ui| {
                                                if issue.status() == &Status::Open {
                                                    if ui.button("Close as Completed").clicked()
                                                        && let Some(target) =
                                                            self.issues.get_mut(id)
                                                    {
                                                        target.close_as_cmp();
                                                        ui.close_menu();
                                                    }

                                                    if ui.button("Close as Not Planned").clicked()
                                                        && let Some(target) =
                                                            self.issues.get_mut(id)
                                                    {
                                                        target.close_as_not_planed();
                                                        ui.close_menu();
                                                    }
                                                    ui.separator();
                                                } else {
                                                    // Closedの場合に Reopen を表示
                                                    if ui.button("Reopen Issue").clicked()
                                                        && let Some(target) =
                                                            self.issues.get_mut(id)
                                                    {
                                                        target.reopen();
                                                        ui.close_menu();
                                                    }
                                                    ui.separator();
                                                }

                                                if ui.button("Fork this Issue").clicked()
                                                    && let Some(new_id) = self.issues.fork(id)
                                                {
                                                    self.filter_status = FilterStatus::All;
                                                    self.selected_issue_index = Some(new_id);
                                                    ui.close_menu();
                                                }
                                            });
                                        },
                                    );
                                });
                        },
                    );
                } else {
                    ui.centered_and_justified(|ui| {
                        ui.label("Issue not found.");
                    });
                }
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("Select an issue from the list to view details.");
                });
            }
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_opts = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1024.0, 768.0])
            .with_drag_and_drop(true),
        vsync: false,
        ..Default::default()
    };
    eframe::run_native(
        "Ramifi",
        native_opts,
        Box::new(|cc| Ok(Box::new(RamifiApp::new(cc)))),
    )
}
