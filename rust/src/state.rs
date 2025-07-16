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
            range_manager: Mutex::new(Default::default()),
            action_tree: Mutex::new(default_action_tree()),
            bunching_data: Mutex::new(None),
            post_flop_game: Mutex::new(Default::default()),
            thread_pool: Mutex::new(ThreadPoolBuilder::new().build().unwrap()),
        }
    }
}

impl SessionState {
    pub fn reset(&self) {
        *self.range_manager.lock() = Default::default();
        *self.action_tree.lock() = default_action_tree();
        *self.bunching_data.lock() = None;
        *self.post_flop_game.lock() = Default::default();
        *self.thread_pool.lock() = ThreadPoolBuilder::new().build().unwrap();
    }
}
