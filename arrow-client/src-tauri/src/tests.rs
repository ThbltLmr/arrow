#[cfg(test)]
mod tests {
    use super::*;
    use crate::db_manager::DbManager;
    use crate::postures::Posture;

    #[test]
    fn test_posture_enum_conversion() {
        let posture = Posture::Straight;
        assert_eq!(posture.get_posture_value(), "STRAIGHT");
        assert_eq!(posture.get_posture_message(), "Straight");

        let bad_posture = Posture::SlouchingBack;
        assert_eq!(bad_posture.get_posture_value(), "SLOUCHING_BACK");
        assert_eq!(bad_posture.get_posture_message(), "Slouching back");
    }

    #[test]
    fn test_posture_from_string() {
        let posture = Posture::from("STRAIGHT");
        assert!(matches!(posture, Posture::Straight));

        let unknown_posture = Posture::from("INVALID");
        assert!(matches!(unknown_posture, Posture::Unknown));
    }

    #[tokio::test]
    async fn test_database_operations() {
        // Create a temporary database for testing
        let db_manager = DbManager::new().expect("Failed to create database");
        
        // Test session start
        assert!(db_manager.log_session_start().is_ok());

        // Test posture change logging
        assert!(db_manager.log_posture_change("STRAIGHT", "SLOUCHING_BACK").is_ok());
        assert!(db_manager.log_posture_change("SLOUCHING_BACK", "STRAIGHT").is_ok());

        // Test session end
        assert!(db_manager.log_session_end("STRAIGHT").is_ok());
    }
}