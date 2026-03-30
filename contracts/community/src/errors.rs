use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CommunityError {
    Unauthorized = 100,
    ForumCategoryNotFound = 101,
    PostNotFound = 102,
    CommentNotFound = 103,
    ArticleNotFound = 104,
    EventNotFound = 105,
    EventFull = 106,
    FeedbackNotFound = 107,
    MentorshipNotFound = 108,
    InsufficientReputation = 109,
    InvalidRating = 110,
    AlreadExists = 111,
}
