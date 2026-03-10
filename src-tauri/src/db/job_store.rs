use super::{Database, DbError};
use crate::jobs::{Job, JobLog, JobStatus, JobType, LogLevel};
use chrono::Utc;
use rusqlite::params;

fn parse_job_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Job> {
    let status_str: String = row.get(2)?;
    let metadata_json: Option<String> = row.get(10)?;
    Ok(Job {
        id: row.get(0)?,
        job_type: row
            .get::<_, String>(1)?
            .parse::<JobType>()
            .unwrap_or(JobType::Custom("unknown".into())),
        status: status_str.parse::<JobStatus>().unwrap_or(JobStatus::Queued),
        created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(4)?)
            .map(|t| t.with_timezone(&Utc))
            .unwrap_or_else(|_| Utc::now()),
        started_at: row
            .get::<_, Option<String>>(5)?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|t| t.with_timezone(&Utc)),
        completed_at: row
            .get::<_, Option<String>>(6)?
            .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
            .map(|t| t.with_timezone(&Utc)),
        progress: row.get(7)?,
        progress_message: row.get(8)?,
        error: row.get(9)?,
        metadata: metadata_json.and_then(|s| serde_json::from_str(&s).ok()),
    })
}

impl Database {
    /// Create a new job
    pub fn create_job(&self, job: &Job) -> Result<(), DbError> {
        let metadata_json = job.metadata.as_ref().map(|m| m.to_string());
        self.conn.execute(
            "INSERT INTO jobs (id, job_type, status, created_at, updated_at, started_at, completed_at,
                    progress, progress_message, error, metadata_json)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                job.id,
                job.job_type.to_string(),
                job.status.to_string(),
                job.created_at.to_rfc3339(),
                job.updated_at.to_rfc3339(),
                job.started_at.map(|t| t.to_rfc3339()),
                job.completed_at.map(|t| t.to_rfc3339()),
                job.progress,
                job.progress_message,
                job.error,
                metadata_json,
            ],
        )?;
        Ok(())
    }

    /// Get a job by ID
    pub fn get_job(&self, job_id: &str) -> Result<Option<Job>, DbError> {
        match self.conn.query_row(
            "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                    progress, progress_message, error, metadata_json
             FROM jobs WHERE id = ?",
            [job_id],
            parse_job_row,
        ) {
            Ok(job) => Ok(Some(job)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(DbError::Sqlite(e)),
        }
    }

    /// List jobs, optionally filtered by status
    pub fn list_jobs(&self, status: Option<JobStatus>, limit: usize) -> Result<Vec<Job>, DbError> {
        let jobs = match status {
            Some(status_filter) => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                            progress, progress_message, error, metadata_json
                     FROM jobs WHERE status = ? ORDER BY created_at DESC LIMIT ?",
                )?;
                let jobs = stmt
                    .query_map(params![status_filter.to_string(), limit as i64], parse_job_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                jobs
            }
            None => {
                let mut stmt = self.conn.prepare(
                    "SELECT id, job_type, status, created_at, updated_at, started_at, completed_at,
                            progress, progress_message, error, metadata_json
                     FROM jobs ORDER BY created_at DESC LIMIT ?",
                )?;
                let jobs = stmt
                    .query_map(params![limit as i64], parse_job_row)?
                    .collect::<Result<Vec<_>, _>>()?;
                jobs
            }
        };

        Ok(jobs)
    }

    /// Update job status
    pub fn update_job_status(
        &self,
        job_id: &str,
        status: JobStatus,
        error: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        let started_at = if status == JobStatus::Running {
            Some(now.clone())
        } else {
            None
        };
        let completed_at = if status.is_terminal() {
            Some(now.clone())
        } else {
            None
        };

        self.conn.execute(
            "UPDATE jobs SET status = ?, updated_at = ?, started_at = COALESCE(?, started_at),
                    completed_at = COALESCE(?, completed_at), error = COALESCE(?, error)
             WHERE id = ?",
            params![
                status.to_string(),
                now,
                started_at,
                completed_at,
                error,
                job_id
            ],
        )?;
        Ok(())
    }

    /// Update job progress
    pub fn update_job_progress(
        &self,
        job_id: &str,
        progress: f32,
        message: Option<&str>,
    ) -> Result<(), DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE jobs SET progress = ?, progress_message = ?, updated_at = ? WHERE id = ?",
            params![progress, message, now, job_id],
        )?;
        Ok(())
    }

    /// Add a log entry for a job
    pub fn add_job_log(
        &self,
        job_id: &str,
        level: LogLevel,
        message: &str,
    ) -> Result<i64, DbError> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO job_logs (job_id, timestamp, level, message) VALUES (?, ?, ?, ?)",
            params![job_id, now, level.to_string(), message],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    /// Get logs for a job
    pub fn get_job_logs(&self, job_id: &str, limit: usize) -> Result<Vec<JobLog>, DbError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, job_id, timestamp, level, message
             FROM job_logs WHERE job_id = ? ORDER BY timestamp DESC LIMIT ?",
        )?;

        let logs = stmt
            .query_map(params![job_id, limit as i64], |row| {
                let level_str: String = row.get(3)?;
                let level = match level_str.as_str() {
                    "debug" => LogLevel::Debug,
                    "info" => LogLevel::Info,
                    "warning" => LogLevel::Warning,
                    "error" => LogLevel::Error,
                    _ => LogLevel::Info,
                };
                Ok(JobLog {
                    id: row.get(0)?,
                    job_id: row.get(1)?,
                    timestamp: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                        .map(|t| t.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                    level,
                    message: row.get(4)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(logs)
    }

    /// Delete old completed jobs
    pub fn cleanup_old_jobs(&self, keep_days: i64) -> Result<usize, DbError> {
        let cutoff = (Utc::now() - chrono::Duration::days(keep_days)).to_rfc3339();
        let deleted = self.conn.execute(
            "DELETE FROM jobs WHERE status IN ('succeeded', 'failed', 'cancelled')
             AND completed_at < ?",
            params![cutoff],
        )?;
        Ok(deleted)
    }

    /// Get count of jobs by status
    pub fn get_job_counts(&self) -> Result<Vec<(String, i64)>, DbError> {
        let mut stmt = self
            .conn
            .prepare("SELECT status, COUNT(*) FROM jobs GROUP BY status")?;

        let counts = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(counts)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::MasterKey;
    use tempfile::tempdir;

    fn create_test_db() -> (Database, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let key = MasterKey::generate();
        let db = Database::open(&db_path, &key).unwrap();
        db.initialize().unwrap();
        (db, dir)
    }

    #[test]
    fn test_job_crud() {
        let (db, _dir) = create_test_db();

        let job = Job::new(JobType::IngestWeb);
        let job_id = job.id.clone();
        db.create_job(&job).unwrap();

        let retrieved = db.get_job(&job_id).unwrap();
        assert!(retrieved.is_some());
        let retrieved = retrieved.unwrap();
        assert_eq!(retrieved.id, job_id);
        assert_eq!(retrieved.status, JobStatus::Queued);

        db.update_job_status(&job_id, JobStatus::Running, None)
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.status, JobStatus::Running);
        assert!(retrieved.started_at.is_some());

        db.update_job_progress(&job_id, 0.5, Some("Halfway done"))
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.progress, 0.5);
        assert_eq!(retrieved.progress_message, Some("Halfway done".to_string()));

        db.update_job_status(&job_id, JobStatus::Succeeded, None)
            .unwrap();
        let retrieved = db.get_job(&job_id).unwrap().unwrap();
        assert_eq!(retrieved.status, JobStatus::Succeeded);
        assert!(retrieved.completed_at.is_some());
    }

    #[test]
    fn test_job_logs() {
        let (db, _dir) = create_test_db();

        let job = Job::new(JobType::IngestYoutube);
        let job_id = job.id.clone();
        db.create_job(&job).unwrap();

        db.add_job_log(&job_id, LogLevel::Info, "Starting ingestion")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Debug, "Fetching content")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Warning, "Content is large")
            .unwrap();
        db.add_job_log(&job_id, LogLevel::Info, "Completed")
            .unwrap();

        let logs = db.get_job_logs(&job_id, 10).unwrap();
        assert_eq!(logs.len(), 4);
        assert_eq!(logs[0].message, "Completed");
        assert_eq!(logs[3].message, "Starting ingestion");
    }

    #[test]
    fn test_list_jobs_by_status() {
        let (db, _dir) = create_test_db();

        let job1 = Job::new(JobType::IngestWeb);
        let job2 = Job::new(JobType::IngestYoutube);
        let job3 = Job::new(JobType::IndexKb);

        db.create_job(&job1).unwrap();
        db.create_job(&job2).unwrap();
        db.create_job(&job3).unwrap();

        db.update_job_status(&job1.id, JobStatus::Running, None)
            .unwrap();
        db.update_job_status(&job2.id, JobStatus::Succeeded, None)
            .unwrap();

        let all_jobs = db.list_jobs(None, 10).unwrap();
        assert_eq!(all_jobs.len(), 3);

        let queued = db.list_jobs(Some(JobStatus::Queued), 10).unwrap();
        assert_eq!(queued.len(), 1);

        let running = db.list_jobs(Some(JobStatus::Running), 10).unwrap();
        assert_eq!(running.len(), 1);

        let succeeded = db.list_jobs(Some(JobStatus::Succeeded), 10).unwrap();
        assert_eq!(succeeded.len(), 1);
    }

    #[test]
    fn test_job_counts() {
        let (db, _dir) = create_test_db();

        let job1 = Job::new(JobType::IngestWeb);
        let job2 = Job::new(JobType::IngestYoutube);
        let job3 = Job::new(JobType::IndexKb);

        db.create_job(&job1).unwrap();
        db.create_job(&job2).unwrap();
        db.create_job(&job3).unwrap();

        db.update_job_status(&job1.id, JobStatus::Succeeded, None)
            .unwrap();
        db.update_job_status(&job2.id, JobStatus::Failed, Some("Test error"))
            .unwrap();

        let counts = db.get_job_counts().unwrap();
        assert!(!counts.is_empty());

        let failed_job = db.get_job(&job2.id).unwrap().unwrap();
        assert_eq!(failed_job.error, Some("Test error".to_string()));
    }
}
