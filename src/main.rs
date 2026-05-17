use macroquad::prelude::*;
use macroquad::audio::{play_sound_once, PlaySoundParams, Sound};
use std::collections::VecDeque;

// 円盤の構造体
struct Disk {
    size: i32,
    color: Color,
    pos: Vec2, // 現在の描画座標
}

// 移動命令を記録する構造体
#[derive(Clone, Copy)]
struct MoveStep {
    from: usize,
    to: usize,
}

// 再帰的に移動手順を生成する関数
fn solve_hanoi(n: i32, from: usize, to: usize, work: usize, steps: &mut Vec<MoveStep>) {
    if n > 0 {
        solve_hanoi(n - 1, from, work, to, steps);
        steps.push(MoveStep { from, to });
        solve_hanoi(n - 1, work, to, from, steps);
    }
}

// 指定した柱・高さから「あるべき座標」を計算する関数
fn get_target_pos(pole_idx: usize, height_idx: usize, center_x: f32) -> Vec2 {
    let pole_width = 200.0;
    let x = (center_x - pole_width) + pole_idx as f32 * pole_width;
    Vec2::new(x, 500.0 - height_idx as f32 * 25.0 - 12.5)
}

// 簡易的なビープ音を生成する（簡易SEの代用。本来はwav読み込みが推奨されます）
async fn play_click_se(size: i32) {
    // macroquad の組み込みオーディオ機能や外部リソースがない場合、
    // 本来は load_sound("click.wav").await を使いますが、ここではダミー関数として定義。
    // 実際のwavファイルを鳴らす場合は、以下のようにPlaySoundParamsでピッチを変更できます。
    /*
    play_sound_once(
        sound,
        PlaySoundParams {
            loose_pitch: true,
            pitch: 1.2 - (size as f32 * 0.1), // 小さい円盤ほど高音
            volume: 1.0,
        },
    );
    */
}

#[macroquad::main("Rust ハノイの塔")]
async fn main() {
    let num_disks = 5;
    let mut poles: Vec<Vec<Disk>> = vec![vec![], vec![], vec![]];
    let center_x = screen_width() / 2.0;

    // 初期化：一番左の柱に円盤をセット
    for i in (1..=num_disks).rev() {
        let hsv_color = Color::from_rgba(
            ((i * 40) % 360) as u8, 
            180, 
            230, 
            255
        ); // 簡易的な色生成
        let target = get_target_pos(0, poles[0].len(), center_x);
        poles[0].push(Disk {
            size: i,
            color: hsv_color,
            pos: target,
        });
    }

    let mut selected_pole: Option<usize> = None;
    
    // 自動解法用の変数
    let mut auto_steps: VecDeque<MoveStep> = VecDeque::new();
    let mut is_auto_solving = false;
    let mut auto_timer = 0.0;
    let step_delay = 0.5; // 0.5秒ごとに1手

    loop {
        let center_x = screen_width() / 2.0;
        let delta_time = get_frame_time();
        clear_background(LIGHTGRAY);

        // --- 1. 自動解法の実行ロジック ---
        if is_auto_solving {
            auto_timer += delta_time;
            if auto_timer >= step_delay {
                if let Some(step) = auto_steps.pop_front() {
                    if !poles[step.from].is_empty() {
                        let disk = poles[step.from].pop().unwrap();
                        // play_click_se(disk.size).await; // SE再生
                        poles[step.to].push(disk);
                    }
                    auto_timer = 0.0;
                } else {
                    is_auto_solving = false;
                }
            }
        }

        // --- 2. 座標の更新 (Lerp アニメーション) ---
        for i in 0..3 {
            let pole_len = poles[i].len();
            for j in 0..pole_len {
                let mut target = get_target_pos(i, j, center_x);
                
                // 選択中の円盤は少し浮かせる
                if selected_pole == Some(i) && j == pole_len - 1 {
                    target.y -= 40.0;
                }

                // Lerp による追従 (Rustでは自前実装、またはマクロの等価式)
                let t = 0.2; // 追従スピード
                poles[i][j].pos.x = poles[i][j].pos.x + (target.x - poles[i][j].pos.x) * t;
                poles[i][j].pos.y = poles[i][j].pos.y + (target.y - poles[i][j].pos.y) * t;
            }
        }

        // --- 3. 描画とクリック判定 ---
        let pole_width = 200.0;
        let pole_x_base = center_x - pole_width;
        let is_clear = poles[2].len() == num_disks as usize;

        for i in 0..3 {
            let x = pole_x_base + i as f32 * pole_width;
            
            // 柱と土台の描画
            draw_rectangle(x - 90.0, 500.0, 180.0, 10.0, GRAY); // 土台
            draw_rectangle(x - 10.0, 200.0, 20.0, 300.0, DARKGRAY); // 柱

            // 円盤の描画
            for (j, disk) in poles[i].iter().enumerate() {
                let w = disk.size as f32 * 30.0;
                let h = 25.0;
                
                // 円盤本体
                draw_rectangle(disk.pos.x - w / 2.0, disk.pos.y - h / 2.0, w, h, disk.color);
                
                // 選択中のハイライト枠
                if selected_pole == Some(i) && j == poles[i].len() - 1 {
                    draw_rectangle_lines(disk.pos.x - w / 2.0, disk.pos.y - h / 2.0, w, h, 4.0, YELLOW);
                } else {
                    draw_rectangle_lines(disk.pos.x - w / 2.0, disk.pos.y - h / 2.0, w, h, 1.0, BLACK);
                }
            }

            // クリック判定 (クリア時・自動実行時は無効化)
            if !is_clear && !is_auto_solving && is_mouse_button_pressed(MouseButton::Left) {
                let mouse_pos = mouse_position();
                let click_rect_x = x - 90.0;
                let click_rect_y = 200.0;

                // 柱のクリックエリア判定
                if mouse_pos.0 >= click_rect_x && mouse_pos.0 <= click_rect_x + 180.0 
                   && mouse_pos.1 >= click_rect_y && mouse_pos.1 <= click_rect_y + 310.0 {
                    
                    match selected_pole {
                        None => {
                            // 何も持っていない時：一番上の円盤を「選択」
                            if !poles[i].is_empty() {
                                selected_pole = Some(i);
                            }
                        }
                        Some(from_idx) => {
                            if from_idx == i {
                                selected_pole = None; // 同じ柱ならキャンセル
                            } else {
                                // ルールチェック
                                let can_move = poles[i].last().map_or(true, |target_disk| {
                                    target_disk.size > poles[from_idx].last().unwrap().size
                                });

                                if can_move {
                                    let disk = poles[from_idx].pop().unwrap();
                                    // play_click_se(disk.size).await; // SE再生
                                    poles[i].push(disk);
                                    selected_pole = None;
                                }
                            }
                        }
                    }
                }
            }
        }

        // --- 4. UI とクリア画面 ---
        if is_clear {
            // 暗転オーバーレイ
            draw_rectangle(0.0, 0.0, screen_width(), screen_height(), Color::new(0.0, 0.0, 0.0, 0.5));
            draw_text("✨ CLEAR! ✨", center_x - 100.0, screen_height() / 2.0, 40.0, YELLOW);
        } else if !is_auto_solving {
            // 自動解法ボタンの表示
            draw_rectangle(20.0, 20.0, 160.0, 40.0, BLUE);
            draw_text("Auto Solve", 45.0, 45.0, 20.0, WHITE);

            let mouse_pos = mouse_position();
            if is_mouse_button_pressed(MouseButton::Left) 
               && mouse_pos.0 >= 20.0 && mouse_pos.0 <= 180.0 
               && mouse_pos.1 >= 20.0 && mouse_pos.1 <= 60.0 {
                
                // 自動解法の手順を生成
                let mut steps = Vec::new();
                solve_hanoi(num_disks, 0, 2, 1, &mut steps);
                auto_steps = VecDeque::from(steps);
                is_auto_solving = true;
                auto_timer = 0.0;
            }
        }

        next_frame().await
    }
}