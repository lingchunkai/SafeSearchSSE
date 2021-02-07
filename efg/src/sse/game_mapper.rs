use crate::sse::TreeplexMapper;

#[derive(Debug)]
pub struct GameMapper {
    pub mapper_leader: TreeplexMapper,
    pub mapper_follower: TreeplexMapper,
}

impl GameMapper {
    pub fn new(mapper_leader: TreeplexMapper, mapper_follower: TreeplexMapper) -> GameMapper {
        GameMapper {
            mapper_leader,
            mapper_follower,
        }
    }
}