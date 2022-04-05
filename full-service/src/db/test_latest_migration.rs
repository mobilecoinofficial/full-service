mod test_latest_migration {
    use crate::json_rpc::api_test_utils::setup;
    use mc_common::logger::{test_with_logger, Logger};
    use std::{fs, io, path::PathBuf, process::Command};

    const NUM_MIGRATIONS: usize = 15;
    // just .gitkeep right now
    const NUM_EXTRA_FILES: usize = 1;

    fn get_migration_folders() -> Vec<PathBuf> {
        // get all of the migrations and sort by path, which will also sort by date
        // because of our naming scheme
        let mut migration_folders = fs::read_dir("migrations")
            .unwrap()
            .map(|res| res.map(|e| e.path()))
            .collect::<Result<Vec<_>, io::Error>>()
            .unwrap();

        migration_folders.sort();

        migration_folders
    }

    #[test_with_logger]
    fn test_latest_migration(logger: Logger) {
        // println!("folders? {:?}", migration_folders);
        let migration_folders = get_migration_folders();

        assert_eq!(migration_folders.len(), NUM_MIGRATIONS + NUM_EXTRA_FILES);

        // move the latest migration to a temporary location
        fs::create_dir("temp-migrations").unwrap();
        let latest_migration = migration_folders.last().unwrap().to_str().unwrap();

        let new_path = format!(
            "temp-migrations/{}",
            latest_migration.replace("migrations/", "")
        );
        fs::rename(latest_migration, new_path).unwrap();

        let migration_folders = get_migration_folders();

        assert_eq!(
            migration_folders.len(),
            NUM_MIGRATIONS + NUM_EXTRA_FILES - 1
        );

        // run tests

        // move latest migration back to migrations folder
        let old_path = format!(
            "migrations/{}",
            latest_migration.replace("temp-migrations/", "")
        );
        fs::rename(new_path, old_path).unwrap();

        fs::remove_dir("temp-migrations").unwrap();

        let migration_folders = get_migration_folders();

        assert_eq!(migration_folders.len(), NUM_MIGRATIONS + NUM_EXTRA_FILES);
    }
}
