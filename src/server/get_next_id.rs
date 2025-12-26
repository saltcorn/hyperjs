use super::NEXT_ID;

pub fn get_next_id() -> u32 {
  let mut id = NEXT_ID.lock().unwrap();
  let current = *id;
  *id = id.wrapping_add(1);
  current
}
