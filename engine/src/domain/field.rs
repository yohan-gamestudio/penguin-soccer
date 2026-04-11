use crate::domain::types::Vec2;

#[derive(Debug, Clone)]
pub struct Goal {
    pub center: Vec2,
    pub width: f64,
    pub team: u8,
}

#[derive(Debug, Clone)]
pub struct Field {
    pub width: f64,
    pub height: f64,
    pub goals: [Goal; 2],
}

impl Field {
    pub fn new(width: f64, height: f64) -> Self {
        let goal_width = height * 0.3;
        let goal_y = height / 2.0;
        Self {
            width,
            height,
            goals: [
                Goal {
                    center: Vec2::new(0.0, goal_y),
                    width: goal_width,
                    team: 0,
                },
                Goal {
                    center: Vec2::new(width, goal_y),
                    width: goal_width,
                    team: 1,
                },
            ],
        }
    }

    pub fn center(&self) -> Vec2 {
        Vec2::new(self.width / 2.0, self.height / 2.0)
    }
}
