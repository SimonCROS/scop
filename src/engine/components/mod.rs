use super::FrameInfo;

pub trait Component {
    fn update(&mut self, frame_info: &FrameInfo);
}
