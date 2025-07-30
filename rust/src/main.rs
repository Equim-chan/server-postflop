mod bunching;
mod range;
mod solver;
mod state;
mod tree;

use crate::state::SessionState;
use std::sync::Arc;

use axum::extract::State;
use axum::routing::post;
use axum::{Json, Router};
use axum_embed::ServeEmbed;
use clap::Parser;
use rayon::ThreadPoolBuilder;
use rust_embed::RustEmbed;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_json::json;
use sysinfo::{System, SystemExt};
use tokio::net::TcpListener;
use tokio::signal;

#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

#[derive(RustEmbed, Clone)]
#[folder = "../dist"]
struct Assets;

#[derive(Serialize, Default)]
struct Response {
    result: Value,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// Host to listen to.
    #[arg(short = 'a', long, default_value = "127.0.0.1")]
    host: String,

    /// Port to listen to.
    #[arg(short, long, default_value_t = 7777)]
    port: u16,
}

#[tokio::main]
async fn main() {
    let Args { host, port } = Args::parse();

    let global_session = Arc::new(SessionState::default());
    let invoke_routes = Router::new()
        .route("/reset", post(reset))
        .route("/os_name", post(os_name))
        .route("/memory", post(memory))
        .route("/set_num_threads", post(set_num_threads))
        .route("/get_num_threads", post(get_num_threads))
        .route("/range_num_combos", post(range_num_combos))
        .route("/range_clear", post(range_clear))
        .route("/range_invert", post(range_invert))
        .route("/range_update", post(range_update))
        .route("/range_from_string", post(range_from_string))
        .route("/range_to_string", post(range_to_string))
        .route("/range_get_weights", post(range_get_weights))
        .route("/range_raw_data", post(range_raw_data))
        .route("/tree_new", post(tree_new))
        .route("/tree_added_lines", post(tree_added_lines))
        .route("/tree_removed_lines", post(tree_removed_lines))
        .route("/tree_invalid_terminals", post(tree_invalid_terminals))
        .route("/tree_actions", post(tree_actions))
        .route("/tree_is_terminal_node", post(tree_is_terminal_node))
        .route("/tree_is_chance_node", post(tree_is_chance_node))
        .route("/tree_back_to_root", post(tree_back_to_root))
        .route("/tree_apply_history", post(tree_apply_history))
        .route("/tree_play", post(tree_play))
        .route("/tree_total_bet_amount", post(tree_total_bet_amount))
        .route("/tree_add_bet_action", post(tree_add_bet_action))
        .route("/tree_remove_current_node", post(tree_remove_current_node))
        .route("/tree_delete_added_line", post(tree_delete_added_line))
        .route("/tree_delete_removed_line", post(tree_delete_removed_line))
        .route("/bunching_init", post(bunching_init))
        .route("/bunching_clear", post(bunching_clear))
        .route("/bunching_progress", post(bunching_progress))
        .route("/game_init", post(game_init))
        .route("/game_private_cards", post(game_private_cards))
        .route("/game_memory_usage", post(game_memory_usage))
        .route(
            "/game_memory_usage_bunching",
            post(game_memory_usage_bunching),
        )
        .route("/game_allocate_memory", post(game_allocate_memory))
        .route("/game_set_bunching", post(game_set_bunching))
        .route("/game_solve_step", post(game_solve_step))
        .route("/game_solve_steps", post(game_solve_steps))
        .route("/game_exploitability", post(game_exploitability))
        .route("/game_finalize", post(game_finalize))
        .route("/game_apply_history", post(game_apply_history))
        .route("/game_total_bet_amount", post(game_total_bet_amount))
        .route("/game_actions_after", post(game_actions_after))
        .route("/game_possible_cards", post(game_possible_cards))
        .route("/game_get_results", post(game_get_results))
        .route("/game_get_chance_reports", post(game_get_chance_reports))
        .with_state(global_session);
    let app = Router::new()
        .fallback_service(ServeEmbed::<Assets>::new())
        .nest("/invoke", invoke_routes);

    eprintln!("http://{host}:{port}/");
    let listener = TcpListener::bind((host, port)).await.unwrap();
    axum::serve(listener, app)
        .with_graceful_shutdown(async { signal::ctrl_c().await.unwrap() })
        .await
        .unwrap();
    eprintln!("shutdown received");
}

async fn reset(State(state): State<Arc<SessionState>>) -> Json<Response> {
    state.reset();
    Json(Default::default())
}

#[cfg(target_os = "windows")]
async fn os_name() -> Json<Response> {
    Json(Response {
        result: "windows".into(),
    })
}

#[cfg(target_os = "macos")]
async fn os_name() -> Json<Response> {
    Json(Response {
        result: "macos".into(),
    })
}

#[cfg(target_os = "linux")]
async fn os_name() -> Json<Response> {
    Json(Response {
        result: "linux".into(),
    })
}

async fn memory() -> Json<Response> {
    let mut system = System::new_all();
    system.refresh_memory();
    let result = (system.available_memory(), system.total_memory());
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct SetNumThreadsRequest {
    num_threads: usize,
}

async fn set_num_threads(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<SetNumThreadsRequest>,
) -> Json<Response> {
    *state.thread_pool.lock() = ThreadPoolBuilder::new()
        .num_threads(req.num_threads)
        .build()
        .unwrap();
    Json(Default::default())
}

async fn get_num_threads(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let num_threads = state.thread_pool.lock().current_num_threads();
    Json(Response {
        result: json!(num_threads),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct PlayerRequest {
    player: usize,
}

async fn range_num_combos(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let result = crate::range::range_num_combos(&range_manager, req.player);
    Json(Response {
        result: json!(result),
    })
}

async fn range_clear(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let mut range_manager = state.range_manager.lock();
    crate::range::range_clear(&mut range_manager, req.player);
    Json(Default::default())
}

async fn range_invert(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let mut range_manager = state.range_manager.lock();
    crate::range::range_invert(&mut range_manager, req.player);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RangeUpdateRequest {
    player: usize,
    row: u8,
    col: u8,
    weight: f32,
}

async fn range_update(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<RangeUpdateRequest>,
) -> Json<Response> {
    let mut range_manager = state.range_manager.lock();
    crate::range::range_update(&mut range_manager, req.player, req.row, req.col, req.weight);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RangeFromStringRequest {
    player: usize,
    str: String,
}

async fn range_from_string(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<RangeFromStringRequest>,
) -> Json<Response> {
    let mut range_manager = state.range_manager.lock();
    let result = crate::range::range_from_string(&mut range_manager, req.player, req.str);
    Json(Response {
        result: json!(result),
    })
}

async fn range_to_string(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let result = crate::range::range_to_string(&range_manager, req.player);
    Json(Response {
        result: json!(result),
    })
}

async fn range_get_weights(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let result = crate::range::range_get_weights(&range_manager, req.player);
    Json(Response {
        result: json!(result),
    })
}

async fn range_raw_data(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<PlayerRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let result = crate::range::range_raw_data(&range_manager, req.player);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TreeNewRequest {
    board_len: i32,
    starting_pot: i32,
    effective_stack: i32,
    donk_option: bool,
    oop_flop_bet: String,
    oop_flop_raise: String,
    oop_turn_bet: String,
    oop_turn_raise: String,
    oop_turn_donk: String,
    oop_river_bet: String,
    oop_river_raise: String,
    oop_river_donk: String,
    ip_flop_bet: String,
    ip_flop_raise: String,
    ip_turn_bet: String,
    ip_turn_raise: String,
    ip_river_bet: String,
    ip_river_raise: String,
    add_allin_threshold: f64,
    force_allin_threshold: f64,
    merging_threshold: f64,
    added_lines: String,
    removed_lines: String,
}

async fn tree_new(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreeNewRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    let result = crate::tree::tree_new(
        &mut action_tree,
        req.board_len,
        req.starting_pot,
        req.effective_stack,
        req.donk_option,
        req.oop_flop_bet,
        req.oop_flop_raise,
        req.oop_turn_bet,
        req.oop_turn_raise,
        req.oop_turn_donk,
        req.oop_river_bet,
        req.oop_river_raise,
        req.oop_river_donk,
        req.ip_flop_bet,
        req.ip_flop_raise,
        req.ip_turn_bet,
        req.ip_turn_raise,
        req.ip_river_bet,
        req.ip_river_raise,
        req.add_allin_threshold,
        req.force_allin_threshold,
        req.merging_threshold,
        req.added_lines,
        req.removed_lines,
    );
    Json(Response {
        result: json!(result),
    })
}

async fn tree_added_lines(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_added_lines(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_removed_lines(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_removed_lines(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_invalid_terminals(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_invalid_terminals(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_actions(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_actions(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_is_terminal_node(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_is_terminal_node(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_is_chance_node(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_is_chance_node(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_back_to_root(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_back_to_root(&mut action_tree);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TreeApplyHistoryRequest {
    line: Vec<String>,
}

async fn tree_apply_history(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreeApplyHistoryRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_apply_history(&mut action_tree, req.line);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TreePlayRequest {
    action: String,
}

async fn tree_play(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreePlayRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    let result = crate::tree::tree_play(&mut action_tree, req.action);
    Json(Response {
        result: json!(result),
    })
}

async fn tree_total_bet_amount(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let action_tree = state.action_tree.lock();
    let result = crate::tree::tree_total_bet_amount(&action_tree);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TreeAddBetActionRequest {
    amount: i32,
    is_raise: bool,
}

async fn tree_add_bet_action(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreeAddBetActionRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_add_bet_action(&mut action_tree, req.amount, req.is_raise);
    Json(Default::default())
}

async fn tree_remove_current_node(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_remove_current_node(&mut action_tree);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TreeDeleteLineRequest {
    line: String,
}

async fn tree_delete_added_line(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreeDeleteLineRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_delete_added_line(&mut action_tree, req.line);
    Json(Default::default())
}

async fn tree_delete_removed_line(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<TreeDeleteLineRequest>,
) -> Json<Response> {
    let mut action_tree = state.action_tree.lock();
    crate::tree::tree_delete_removed_line(&mut action_tree, req.line);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BunchingInitRequest {
    board: Vec<u8>,
}

async fn bunching_init(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<BunchingInitRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let mut bunching_data = state.bunching_data.lock();
    let result = crate::bunching::bunching_init(&range_manager, &mut bunching_data, req.board);
    Json(Response {
        result: json!(result),
    })
}

async fn bunching_clear(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut bunching_data = state.bunching_data.lock();
    crate::bunching::bunching_clear(&mut bunching_data);
    Json(Default::default())
}

async fn bunching_progress(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut bunching_data = state.bunching_data.lock();
    let thread_pool = state.thread_pool.lock();
    let result = crate::bunching::bunching_progress(&mut bunching_data, &thread_pool);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameInitRequest {
    board: Vec<u8>,
    starting_pot: i32,
    effective_stack: i32,
    rake_rate: f64,
    rake_cap: f64,
    donk_option: bool,
    oop_flop_bet: String,
    oop_flop_raise: String,
    oop_turn_bet: String,
    oop_turn_raise: String,
    oop_turn_donk: String,
    oop_river_bet: String,
    oop_river_raise: String,
    oop_river_donk: String,
    ip_flop_bet: String,
    ip_flop_raise: String,
    ip_turn_bet: String,
    ip_turn_raise: String,
    ip_river_bet: String,
    ip_river_raise: String,
    add_allin_threshold: f64,
    force_allin_threshold: f64,
    merging_threshold: f64,
    added_lines: String,
    removed_lines: String,
}

async fn game_init(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameInitRequest>,
) -> Json<Response> {
    let range_manager = state.range_manager.lock();
    let mut post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_init(
        &range_manager,
        &mut post_flop_game,
        req.board,
        req.starting_pot,
        req.effective_stack,
        req.rake_rate,
        req.rake_cap,
        req.donk_option,
        req.oop_flop_bet,
        req.oop_flop_raise,
        req.oop_turn_bet,
        req.oop_turn_raise,
        req.oop_turn_donk,
        req.oop_river_bet,
        req.oop_river_raise,
        req.oop_river_donk,
        req.ip_flop_bet,
        req.ip_flop_raise,
        req.ip_turn_bet,
        req.ip_turn_raise,
        req.ip_river_bet,
        req.ip_river_raise,
        req.add_allin_threshold,
        req.force_allin_threshold,
        req.merging_threshold,
        req.added_lines,
        req.removed_lines,
    );
    Json(Response {
        result: json!(result),
    })
}

async fn game_private_cards(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_private_cards(&post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

async fn game_memory_usage(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_memory_usage(&post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

async fn game_memory_usage_bunching(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_memory_usage_bunching(&post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameAllocateMemoryRequest {
    enable_compression: bool,
}

async fn game_allocate_memory(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameAllocateMemoryRequest>,
) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    crate::solver::game_allocate_memory(&mut post_flop_game, req.enable_compression);
    Json(Default::default())
}

async fn game_set_bunching(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let bunching_data = state.bunching_data.lock();
    let mut post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_set_bunching(&bunching_data, &mut post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameSolveStepRequest {
    current_iteration: u32,
}

async fn game_solve_step(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameSolveStepRequest>,
) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let thread_pool = state.thread_pool.lock();
    crate::solver::game_solve_step(&post_flop_game, &thread_pool, req.current_iteration);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameSolveStepsRequest {
    current_iteration: u32,
    num_iterations: u32,
}

async fn game_solve_steps(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameSolveStepsRequest>,
) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let thread_pool = state.thread_pool.lock();
    crate::solver::game_solve_steps(
        &post_flop_game,
        &thread_pool,
        req.current_iteration,
        req.num_iterations,
    );
    Json(Default::default())
}

async fn game_exploitability(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let thread_pool = state.thread_pool.lock();
    let result = crate::solver::game_exploitability(&post_flop_game, &thread_pool);
    Json(Response {
        result: json!(result),
    })
}

async fn game_finalize(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    let thread_pool = state.thread_pool.lock();
    crate::solver::game_finalize(&mut post_flop_game, &thread_pool);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameApplyHistoryRequest {
    history: Vec<usize>,
}

async fn game_apply_history(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameApplyHistoryRequest>,
) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    crate::solver::game_apply_history(&mut post_flop_game, req.history);
    Json(Default::default())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameTotalBetAmountRequest {
    append: Vec<isize>,
}

async fn game_total_bet_amount(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameTotalBetAmountRequest>,
) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_total_bet_amount(&mut post_flop_game, req.append);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameActionsAfterRequest {
    append: Vec<isize>,
}

async fn game_actions_after(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameActionsAfterRequest>,
) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_actions_after(&mut post_flop_game, req.append);
    Json(Response {
        result: json!(result),
    })
}

async fn game_possible_cards(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_possible_cards(&post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

async fn game_get_results(State(state): State<Arc<SessionState>>) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    let result = crate::solver::game_get_results(&mut post_flop_game);
    Json(Response {
        result: json!(result),
    })
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GameGetChanceReportsRequest {
    append: Vec<isize>,
    num_actions: usize,
}

async fn game_get_chance_reports(
    State(state): State<Arc<SessionState>>,
    Json(req): Json<GameGetChanceReportsRequest>,
) -> Json<Response> {
    let mut post_flop_game = state.post_flop_game.lock();
    let result =
        crate::solver::game_get_chance_reports(&mut post_flop_game, req.append, req.num_actions);
    Json(Response {
        result: json!(result),
    })
}
