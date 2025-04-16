//! Library for communicating about "to-do" objects with a database.
//!
//! See [`TodoTask`] for usage.

#![deny(clippy::pedantic)]
#![deny(missing_docs)]

use chrono::{DateTime, TimeZone, Utc};
use serde::{Deserialize, Serialize};

/// Status of a "to-do" item.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
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
/// Create a new task with [`TodoTask::new`]:
///
/// ```
/// use chrono::{TimeDelta, Utc};
/// use dts_developer_challenge::{TodoTask, TodoStatus};
///
/// // create a due date twelve hours from now
/// let due = Utc::now() + TimeDelta::hours(12);
/// let task = TodoTask::new(
///     "My title".to_string(),
///     Some("My description".to_string()),
///     TodoStatus::InProgress,
///     &due,
/// );
/// ```
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
    ///
    /// # Panics
    ///
    /// Panics if the above invariants are not upheld.
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

    /// Check if this task is past due.
    #[must_use]
    pub fn past_due(&self) -> bool {
        self.due < Utc::now()
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeDelta;
    use rstest::*;

    use super::*;

    #[fixture]
    pub fn sample_task() -> TodoTask {
        let due = Utc::now() + TimeDelta::hours(12);
        TodoTask::new("my title".to_string(), None, TodoStatus::InProgress, &due)
    }

    #[rstest]
    fn set_title(mut sample_task: TodoTask) {
        let new_title = "Another new title!";
        sample_task.set_title(new_title.to_string());
        assert_eq!(sample_task.title(), new_title);
    }

    #[rstest]
    #[should_panic]
    fn empty_title(mut sample_task: TodoTask) {
        sample_task.set_title(String::new());
    }

    #[rstest]
    fn set_description(mut sample_task: TodoTask) {
        let new_description = "Another new description!";
        sample_task.set_description(Some(new_description.to_string()));
        assert_eq!(sample_task.description(), Some(new_description));
    }

    #[rstest]
    #[should_panic]
    fn empty_description(mut sample_task: TodoTask) {
        sample_task.set_description(Some(String::new()));
    }

    #[rstest]
    fn set_due(mut sample_task: TodoTask) {
        let new_due = Utc::now() + TimeDelta::days(1);
        sample_task.set_due(&new_due);
        assert_eq!(sample_task.due(), &new_due);
    }

    #[rstest]
    fn past_due(mut sample_task: TodoTask) {
        sample_task.set_due(&(Utc::now() - TimeDelta::days(1)));
        assert!(sample_task.past_due());

        sample_task.set_due(&(Utc::now() + TimeDelta::days(1)));
        assert!(!sample_task.past_due());
    }
}
