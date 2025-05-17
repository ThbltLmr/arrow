pub enum Posture {
    ShouldersNotVisible,
    HeadNotVisible,
    SlouchingBack,
    LeaningIn,
    HeadTiltLeft,
    HeadTiltRight,
    BodyTiltLeft,
    BodyTiltRight,
    Straight,
    Unknown,
}

fn get_posture_value(value: Posture) -> String {
    match value {
        Posture::ShouldersNotVisible => "SHOULDERS_NOT_VISIBLE".to_string(),
        Posture::HeadNotVisible => "HEAD_NOT_VISIBLE".to_string(),
        Posture::SlouchingBack => "SLOUCHING_BACK".to_string(),
        Posture::LeaningIn => "LEANING_IN".to_string(),
        Posture::HeadTiltLeft => "HEAD_TILT_LEFT".to_string(),
        Posture::HeadTiltRight => "HEAD_TILT_RIGHT".to_string(),
        Posture::BodyTiltLeft => "BODY_TILT_LEFT".to_string(),
        Posture::BodyTiltRight => "BODY_TILT_RIGHT".to_string(),
        Posture::Straight => "STRAIGHT".to_string(),
        Posture::Unknown => "UNKNOWN".to_string(),
    }
}

fn get_posture_message(value: Posture) -> String {
    match value {
        Posture::ShouldersNotVisible => "Shoulders not visible".to_string(),
        Posture::HeadNotVisible => "Head not visible".to_string(),
        Posture::SlouchingBack => "Slouching back".to_string(),
        Posture::LeaningIn => "Leaning in".to_string(),
        Posture::HeadTiltLeft => "Head tilt left".to_string(),
        Posture::HeadTiltRight => "Head tilt right".to_string(),
        Posture::BodyTiltLeft => "Body tilt left".to_string(),
        Posture::BodyTiltRight => "Body tilt right".to_string(),
        Posture::Straight => "Straight".to_string(),
        Posture::Unknown => "Unknown".to_string(),
    }
}
