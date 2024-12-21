use uuid::Uuid;

pub struct LimitOrderParams {
    pub asset: String,
    pub is_buy: bool,
    pub price: f64,
    pub size: f64,
    pub reduce_only: Option<bool>,
    pub time_in_force: Option<String>,
    pub cloid: Option<Uuid>,
}

impl LimitOrderParams {
    pub fn new(asset: String, is_buy: bool, price: f64, size: f64) -> Self {
        Self {
            asset,
            is_buy,
            price,
            size,
            reduce_only: None,
            time_in_force: None,
            cloid: None,
        }
    }

    pub fn reduce_only(mut self, value: bool) -> Self {
        self.reduce_only = Some(value);
        self
    }

    pub fn time_in_force(mut self, value: String) -> Self {
        self.time_in_force = Some(value);
        self
    }

    pub fn cloid(mut self, value: Uuid) -> Self {
        self.cloid = Some(value);
        self
    }
}
