use crate::range::RangeManager;
use crate::tree::default_action_tree;

use parking_lot::Mutex;
use postflop_solver::{ActionTree, BunchingData, PostFlopGame};
use rayon::{ThreadPool, ThreadPoolBuilder};

pub struct SessionState {
    pub range_manager: Mutex<RangeManager>,
    pub action_tree: Mutex<ActionTree>,
    pub bunching_data: Mutex<Option<BunchingData>>,
    pub post_flop_game: Mutex<PostFlopGame>,
    pub thread_pool: Mutex<ThreadPool>,
}

impl Default for SessionState {
    fn default() -> Self {
        Self {
            range_manager: Mutex::new(RangeManager::default()),
            action_tree: Mutex::new(default_action_tree()),
            bunching_data: Mutex::new(None),
            post_flop_game: Mutex::new(PostFlopGame::default()),
            thread_pool: Mutex::new(ThreadPoolBuilder::new().build().unwrap()),
        }
    }
}
