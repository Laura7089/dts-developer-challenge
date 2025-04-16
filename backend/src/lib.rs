//! Library for communicating about "to-do" objects with a database.
//!
//! See [`TodoTask`] for usage.

#![warn(clippy::pedantic)]
#![deny(missing_docs)]

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Status of a "to-do" item.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub enum TodoStatus {
    /// Not yet started.
    ///
    /// This is the default value with [`Default::default`].
    #[default]
    NotStarted,
    /// Currently being worked on.
    InProgress,
    /// Finished 🎉 !
    Complete,
    /// Will not be completed.
    Cancelled,
    /// Cannot be started due to external circumstances.
    Blocked,
}

/// "To-do" task.
///
/// Create a new one with [`TodoTask::new`].
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TodoTask {
    /// Title of the task.
    ///
    /// It is illegal for this to be empty.
    title: String,
    /// In-Depth description of the task.
    ///
    /// If `Some`, it is illegal for this to be empty.
    description: Option<String>,
    /// Current status of the task.
    pub status: TodoStatus,
    /// Date & time at which the task is due, in UTC.
    ///
    /// UTC is the state that the time is stored in memory and the database.
    due: DateTime<Utc>,
}

impl TodoTask {
    /// Create a new [`TodoTask`].
    ///
    /// Requirements of arguments:
    /// - `title` may not be empty
    /// - `description` may not be `Some` *and* empty
    // TODO: builder API?
    pub fn new<TZ: TimeZone>(
        title: String,
        description: Option<String>,
        status: TodoStatus,
        due: &DateTime<TZ>,
    ) -> Self {
        let mut to_return = Self {
            // we can set `title` to an invalid value here because it will
            // always be replaced by the .set_title call
            title: String::new(),
            description: None,
            status,
            due: Utc::now(),
        };

        // use setters for DRY with upholding our invariants
        to_return.set_title(title);
        to_return.set_description(description);
        to_return.set_due(due);

        to_return
    }

    /// Get the title of the task.
    #[must_use]
    pub fn title(&self) -> &str {
        &self.title
    }

    /// Set the title of the task.
    ///
    /// `new_title` *must* not be the empty string.
    ///
    /// # Panics
    ///
    /// Panics when `new_title` is empty.
    pub fn set_title(&mut self, new_title: String) {
        debug_assert!(!new_title.is_empty());

        self.title = new_title;
    }

    /// Get the description of the task.
    ///
    /// The description can never be `Some("")`.
    #[must_use]
    pub fn description(&self) -> Option<&str> {
        self.description.as_deref()
    }

    /// Set the description of the task.
    ///
    /// # Panics
    ///
    /// Panics if `new_description` is `Some("")`.
    pub fn set_description(&mut self, new_description: Option<String>) {
        debug_assert!(!matches!(new_description.as_deref(), Some("")));

        self.description = new_description;
    }

    /// Get the due date & time of the task.
    #[must_use]
    pub fn due(&self) -> &DateTime<Utc> {
        &self.due
    }

    /// Set the due date of the task.
    ///
    /// This method is generic over timezones with `TZ`.
    /// Time zone conversion is performed automatically.
    pub fn set_due<TZ: TimeZone>(&mut self, new_due: &DateTime<TZ>) {
        self.due = new_due.with_timezone(&Utc);
    }
}
