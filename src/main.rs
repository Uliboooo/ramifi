use eframe::egui;
use local_issues_lib::{Comment, Issue, Issues, Status, user::User};

// フィルタリング用のステータス定義
#[derive(PartialEq)]
enum FilterStatus {
    Open,
    Closed,
    All,
}

impl FilterStatus {
    fn matches(&self, status: &Status) -> bool {
        match (self, status) {
            (FilterStatus::Open, Status::Open) => true,
            (FilterStatus::Open, _) => false,
            (FilterStatus::Closed, Status::Open) => false,
            (FilterStatus::Closed, _) => true,
            (FilterStatus::All, _) => true,
        }
    }
}

struct RamifiApp {
    issues: Issues,
    new_description: String,
    new_comment_text: String,
    filter_status: FilterStatus,
    query: String,
    current_user: User,
}

impl RamifiApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Self {
        let current_user = User::new("coyuki", "coyuki@example.com");
        let mut issues = Issues::default();

        // --- Issue 1 ---
        let mut issue1 = Issue::new("GUI実装", current_user.clone(), vec![] as Vec<String>);
        issue1.comment(Comment::new(
            "eframeを使ってGUIを作る",
            current_user.clone(),
        ));
        issue1.comment(Comment::new(
            "local_issues_libとeGUIを統合したGUIアプリケーションを作成",
            current_user.clone(),
        ));
        issues.add_new_issue(issue1);

        // --- Issue 2 ---
        let mut issue2 = Issue::new("永続化", current_user.clone(), vec![] as Vec<String>);
        issue2.comment(Comment::new("データを保存する", current_user.clone()));
        issue2.comment(Comment::new(
            "easy_storageを使った永続化機能を強化する",
            current_user.clone(),
        ));
        issues.add_new_issue(issue2);

        // --- Issue 3 ---
        let mut issue3 = Issue::new("UI改善", current_user.clone(), vec![] as Vec<String>);
        issue3.comment(Comment::new("見た目を良くする", current_user.clone()));
        issue3.comment(Comment::new(
            "モダンなデザインとスムーズな操作感を実現",
            current_user.clone(),
        ));
        issues.add_new_issue(issue3);

        Self {
            issues,
            new_description: String::new(),
            new_comment_text: String::new(),
            filter_status: FilterStatus::Open,
            query: String::new(),
            current_user,
        }
    }
}

impl eframe::App for RamifiApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Ramifi Issue Tracker");

            // --- 新規作成エリア ---
            ui.horizontal(|ui| {
                ui.label("New Issue:");
                ui.text_edit_singleline(&mut self.new_description);
                if ui.button("Add").clicked() {
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

            // --- フィルタリングと検索 ---
            ui.horizontal(|ui| {
                ui.label("Filter:");
                if ui
                    .radio_value(&mut self.filter_status, FilterStatus::Open, "Open")
                    .clicked()
                {};
                if ui
                    .radio_value(&mut self.filter_status, FilterStatus::Closed, "Closed")
                    .clicked()
                {};
                if ui
                    .radio_value(&mut self.filter_status, FilterStatus::All, "All")
                    .clicked()
                {};

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
                .map(|(i, issue): (usize, &Issue)| (i, issue.clone()))
                .collect();

            // 修正箇所 1: .state -> .status() に変更 (Status Enum型へのアクセサと推測)
            display_issues.retain(|(_, issue): &(_, _)| self.filter_status.matches(issue.status()));

            // 検索クエリでフィルタ
            if !self.query.is_empty() {
                let query = self.query.to_lowercase();
                // 修正箇所 2: .title -> .name() に変更 (.title()廃止のため、name()が代替と推測)
                display_issues
                    .retain(|(_, issue): &(_, _)| issue.name().to_lowercase().contains(&query));
            }

            // スクロールエリア
            egui::ScrollArea::vertical().show(ui, |ui| {
                for (id, issue) in display_issues {
                    ui.group(|ui| {
                        // 修正箇所 3: .title -> .name()
                        ui.heading(egui::RichText::new(format!("#{} {}", id + 1, issue.name())));

                        ui.horizontal(|ui| {
                            // 修正箇所 4: .state -> .status()
                            let (status_text, status_color) = match issue.status() {
                                Status::Open => ("Open", egui::Color32::GREEN),
                                _ => ("Closed", egui::Color32::RED),
                            };
                            ui.colored_label(status_color, status_text);

                            // 修正箇所 5: .created_at() (前回修正済み)
                            ui.label(egui::RichText::new(issue.created_at().to_string()).weak());
                        });

                        // コメント追加用UI
                        ui.horizontal(|ui| {
                            ui.text_edit_singleline(&mut self.new_comment_text);
                            if ui.button("Comment").clicked()
                                && let Some(target_issue) = self.issues.get_mut(id)
                            {
                                target_issue.comment(Comment::new(
                                    self.new_comment_text.clone(),
                                    self.current_user.clone(),
                                ));
                                self.new_comment_text.clear();
                            }
                        });
                    });
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
