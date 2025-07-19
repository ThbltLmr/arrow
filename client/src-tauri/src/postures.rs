#[derive(Debug, Clone)]
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
impl Posture {
    pub fn get_posture_value(&self) -> String {
        match &self {
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

    pub fn get_posture_message(&self) -> String {
        match &self {
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
}

impl From<String> for Posture {
    fn from(value: String) -> Self {
        match value.as_str() {
            "SHOULDERS_NOT_VISIBLE" => Posture::ShouldersNotVisible,
            "HEAD_NOT_VISIBLE" => Posture::HeadNotVisible,
            "SLOUCHING_BACK" => Posture::SlouchingBack,
            "LEANING_IN" => Posture::LeaningIn,
            "HEAD_TILT_LEFT" => Posture::HeadTiltLeft,
            "HEAD_TILT_RIGHT" => Posture::HeadTiltRight,
            "BODY_TILT_LEFT" => Posture::BodyTiltLeft,
            "BODY_TILT_RIGHT" => Posture::BodyTiltRight,
            "STRAIGHT" => Posture::Straight,
            "UNKNOWN" => Posture::Unknown,
            _ => Posture::Unknown,
        }
    }
}

impl From<&str> for Posture {
    fn from(value: &str) -> Self {
        match value {
            "SHOULDERS_NOT_VISIBLE" => Posture::ShouldersNotVisible,
            "HEAD_NOT_VISIBLE" => Posture::HeadNotVisible,
            "SLOUCHING_BACK" => Posture::SlouchingBack,
            "LEANING_IN" => Posture::LeaningIn,
            "HEAD_TILT_LEFT" => Posture::HeadTiltLeft,
            "HEAD_TILT_RIGHT" => Posture::HeadTiltRight,
            "BODY_TILT_LEFT" => Posture::BodyTiltLeft,
            "BODY_TILT_RIGHT" => Posture::BodyTiltRight,
            "STRAIGHT" => Posture::Straight,
            "UNKNOWN" => Posture::Unknown,
            _ => Posture::Unknown,
        }
    }
}
