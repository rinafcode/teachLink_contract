pub mod interfaces {
    pub mod itoken;
    pub mod IMarketplace;
    pub mod igovernance;
    pub mod icollaborative_learning;
}

pub mod token;
pub mod marketplace;
pub mod governance;
pub mod collaborative_learning;

// Subscription management system
pub mod subscriptions {
    pub mod interfaces {
        pub mod ISubscriptionManager;
    }
    pub mod libraries {
        pub mod BillingCalculations;
        pub mod PerformanceOptimizations;
    }
    pub mod config;
    pub mod UsageTracker;
    pub mod SubscriptionManager;
}

pub mod types {
    pub mod status;
    pub mod vote_types;
    pub mod learning_types;
}
