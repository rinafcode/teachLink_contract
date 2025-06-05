pub mod contribution_types {
    pub const ASSIGNMENT: u8 = 0;
    pub const RESEARCH: u8 = 1;
    pub const PRESENTATION: u8 = 2;
    pub const OTHER: u8 = 3;
}

pub mod achievement_types {
    pub const COMPLETION: u8 = 0;
    pub const EXCELLENCE: u8 = 1;
    pub const COLLABORATION: u8 = 2;
    pub const INNOVATION: u8 = 3;
}

pub mod dispute_status {
    pub const PENDING: u8 = 0;
    pub const VOTING: u8 = 1;
    pub const RESOLVED_FAVOR: u8 = 2;
    pub const RESOLVED_AGAINST: u8 = 3;
    pub const EXPIRED: u8 = 4;
}

pub mod group_status {
    pub const ACTIVE: u8 = 0;
    pub const INACTIVE: u8 = 1;
    pub const COMPLETED: u8 = 2;
    pub const SUSPENDED: u8 = 3;
}
