use crate::config::StackingMode;

/// Calculates positions for stacked notifications
pub struct StackingLayout;

impl StackingLayout {
    /// Calculate the offset for a notification at the given index
    pub fn calculate_offset(
        mode: &StackingMode,
        index: usize,
        notification_height: i32,
        gap: i32,
    ) -> (i32, i32) {
        match mode {
            StackingMode::Vertical => {
                let y_offset = (index as i32) * (notification_height + gap);
                (0, y_offset)
            }
            StackingMode::Horizontal => {
                // For horizontal stacking, we'd need notification width
                let x_offset = (index as i32) * (350 + gap); // Using default width
                (x_offset, 0)
            }
            StackingMode::Overlay => {
                // All notifications at the same position
                (0, 0)
            }
        }
    }
}
