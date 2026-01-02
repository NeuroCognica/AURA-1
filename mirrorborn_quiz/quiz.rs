use crate::store::AnswerRecord;

pub const TOTAL_PROBES: usize = 240;

pub fn is_complete(answered_count: u64) -> bool {
    (answered_count as usize) >= TOTAL_PROBES
}

pub fn profile_name(session_id: &str) -> String {
    format!("profile_{}.json", session_id)
}
