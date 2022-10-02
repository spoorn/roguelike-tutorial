use bounded_vec_deque::BoundedVecDeque;

pub struct GameLog {
    pub entries: BoundedVecDeque<String>,
}
