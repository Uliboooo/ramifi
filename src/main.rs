use eframe::egui;
use local_issues_lib::{
    Comment, Issue, Issues, Status,
    user::{User, Users},
};
use std::collections::HashMap;
use std::fs;

// ----------------------------------------------------------------------------
// 1. フィルタリング定義
// ----------------------------------------------------------------------------
#[derive(PartialEq)]
enum FilterStatus {
    Open,
    Closed,     // "CloseAsCmp" (完了) を対象
    NotPlanned, // "CloseAsNotPlaned" (計画なし) を対象
    All,
}

impl FilterStatus {
    fn matches(&self, status: &Status) -> bool {
        match (self, status) {
            (FilterStatus::Open, Status::Open) => true,
            (FilterStatus::Open, _) => false,

            (FilterStatus::Closed, Status::CloseAsCmp) => true,
            (FilterStatus::Closed, Status::CloseAsForked) => true,
            (FilterStatus::Closed, _) => false,

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
    scroll_to_issue: Option<usize>, // 追加: ジャンプ先のIssue ID (内部インデックス)

    current_user: User,
}

impl RamifiApp {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // OSの日本語フォント設定
        Self::setup_custom_fonts(&cc.egui_ctx);

        // ユーザー初期化
        let mut users = Users::new();
        let default_user = User::new("coyuki", "coyuki@example.com");
        users.add_user(default_user.clone());
        let current_user = default_user;

        // Issue初期化
        let mut issues = Issues::new();

        // --- 初期データ投入 ---
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
            scroll_to_issue: None,
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
        // --- ユーザーマネージャー ウィンドウ ---
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

                    if ui.button("Create").clicked() {
                        if !self.new_user_name.is_empty() && !self.new_user_email.is_empty() {
                            let new_u = User::new(&self.new_user_name, &self.new_user_email);
                            self.users.add_user(new_u);
                            self.new_user_name.clear();
                            self.new_user_email.clear();
                        }
                    }
                });
        }

        // --- トップパネル ---
        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Current User: {}", self.current_user.name()));
                if ui.button("Manage Users").clicked() {
                    self.show_user_manager = true;
                }
            });
        });

        // --- メインパネル ---
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ramifi Issue Tracker");

            // --- 新規作成 ---
            ui.horizontal(|ui| {
                ui.label("New Issue:");
                ui.text_edit_singleline(&mut self.new_description);
                if ui.button("Add").clicked() && !self.new_description.is_empty() {
                    let mut issue = Issue::new(
                        &self.new_description,
                        self.current_user.clone(),
                        vec![] as Vec<String>,
                    );
                    issue.comment(Comment::new(
                        self.new_description.clone(),
                        self.current_user.clone(),
                    ));
                    self.issues.add_new_issue(issue);
                    self.new_description.clear();
                }
            });

            ui.separator();

            // --- フィルタリング ---
            ui.horizontal(|ui| {
                ui.label("Filter:");
                ui.radio_value(&mut self.filter_status, FilterStatus::Open, "Open");
                ui.radio_value(&mut self.filter_status, FilterStatus::Closed, "Closed");
                ui.radio_value(
                    &mut self.filter_status,
                    FilterStatus::NotPlanned,
                    "Not Planned",
                );
                ui.radio_value(&mut self.filter_status, FilterStatus::All, "All");

                ui.add_space(20.0);
                ui.label("Search:");
                ui.text_edit_singleline(&mut self.query);
            });

            ui.separator();

            // --- リスト表示 ---
            let all_issues = self.issues.get_list();
            let mut display_issues: Vec<(usize, Issue)> = all_issues
                .iter()
                .enumerate()
                .map(|(i, issue)| (i, issue.clone()))
                .collect();

            display_issues.retain(|(_, issue)| self.filter_status.matches(issue.status()));

            if !self.query.is_empty() {
                let query = self.query.to_lowercase();
                display_issues.retain(|(_, issue)| issue.name().to_lowercase().contains(&query));
            }

            egui::ScrollArea::vertical().show(ui, |ui| {
                for (id, issue) in display_issues {
                    // グループ化してレスポンスを取得
                    let group_response = ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.heading(format!("#{} {}", id + 1, issue.name()));

                            let (status_text, status_color) = match issue.status() {
                                Status::Open => ("Open", egui::Color32::GREEN),
                                Status::CloseAsCmp => ("Completed", egui::Color32::RED),
                                Status::CloseAsNotPlaned => ("Not Planned", egui::Color32::GRAY),
                                Status::CloseAsForked => ("Forked", egui::Color32::BLUE),
                            };
                            ui.colored_label(status_color, status_text);

                            // --- Fork元の表示 ---
                            // from_index() が 0 でない場合は Fork された Issue とみなす
                            if issue.from_index() != 0 {
                                // 表示用IDは 内部index + 1
                                let parent_display_id = issue.from_index() + 1;
                                if ui
                                    .link(format!("Forked from #{}", parent_display_id))
                                    .clicked()
                                {
                                    // リンククリック時の動作:
                                    // 1. フィルタを All にして隠れているIssueも表示
                                    self.filter_status = FilterStatus::All;
                                    // 2. スクロールターゲットを設定
                                    self.scroll_to_issue = Some(issue.from_index());
                                }
                            }

                            if ui.button("Fork").clicked() {
                                if let Some(new_id) = self.issues.fork(id) {
                                    println!("Forked issue #{} to #{}", id, new_id);
                                }
                            }

                            ui.menu_button("Close as ...", |ui| {
                                if ui.button("Close (Completed)").clicked() {
                                    if let Some(target) = self.issues.get_mut(id) {
                                        target.close_as_cmp();
                                    }
                                    ui.close_menu();
                                }
                                if ui.button("Close (Not Planned)").clicked() {
                                    if let Some(target) = self.issues.get_mut(id) {
                                        target.close_as_not_planed();
                                    }
                                    ui.close_menu();
                                }
                            });

                            ui.with_layout(
                                egui::Layout::right_to_left(egui::Align::Center),
                                |ui| {
                                    ui.label(
                                        egui::RichText::new(issue.created_at().to_string()).weak(),
                                    );
                                },
                            );
                        });

                        ui.separator();

                        for comment in issue.comments() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(comment.author().name()).strong());
                                ui.label(comment.text());
                            });
                        }

                        ui.separator();

                        ui.horizontal(|ui| {
                            let draft_text = self.comment_drafts.entry(id).or_default();
                            ui.text_edit_singleline(draft_text);

                            if ui.button("Comment").clicked() && !draft_text.is_empty() {
                                if let Some(target_issue) = self.issues.get_mut(id) {
                                    target_issue.comment(Comment::new(
                                        draft_text.clone(),
                                        self.current_user.clone(),
                                    ));
                                    draft_text.clear();
                                }
                            }
                        });
                    });

                    // 自動スクロール処理
                    if self.scroll_to_issue == Some(id) {
                        group_response
                            .response
                            .scroll_to_me(Some(egui::Align::Center));
                        // スクロール完了後にターゲットをリセット
                        self.scroll_to_issue = None;
                    }
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Ramifi",
        native_options,
        Box::new(|cc| Ok(Box::new(RamifiApp::new(cc)))),
    )
}
