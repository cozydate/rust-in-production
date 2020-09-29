
struct IncrementingSource {
    value: u8,
}

impl IncrementingSource {
    pub fn new(value: u8) -> IncrementingSource {
        IncrementingSource { value }
    }

    pub async fn get_and_increment(&mut self) -> u8 {
        let result = self.value;
        self.value += 1;
        result
    }
}

struct Holder {
    value: u8,
}

impl Holder {
    pub fn new() -> Holder {
        Holder { value: 0 }
    }

    pub fn set(&mut self, new_value: u8) {
        self.value = new_value
    }

    pub fn get(&self) -> u8 {
        self.value
    }

    pub fn get_ref(&self) -> &u8 {
        &self.value
    }

    pub fn increment_and_get_ref(&mut self) -> &u8 {
        self.value += 1;
        &self.value
    }

    pub async fn fill(&mut self, source: &mut IncrementingSource) {
        self.value = source.get_and_increment().await;
        self.value = source.get_and_increment().await;
    }

    pub async fn fill_and_get_ref(&mut self, source: &mut IncrementingSource) -> Option<&u8> {
        self.value = source.get_and_increment().await;
        if self.value == 100 {
            return None
        }
        self.value = source.get_and_increment().await;
        Some(&self.value)
    }
}

pub async fn async_main() -> std::io::Result<()> {
    let mut holder = Holder::new();
    holder.set(1);
    {
        let value: &u8 = holder.get_ref();
        println!("value {:?}", value);
    }
    {
        println!("incrementing");
        let value: &u8 = holder.increment_and_get_ref();
        println!("value {:?}", value);
    }
    let mut source = IncrementingSource::new(5);
    {
        println!("fill");
        holder.fill(&mut source).await;
        println!("value {}", holder.get());
    }
    {
        println!("fill_and_get_ref");
        let value: &u8 = holder.fill_and_get_ref(&mut source).await.unwrap();
        println!("value {:?}", value);
    }
    Ok(())
}

pub fn main() {
    tokio::runtime::Runtime::new().unwrap().block_on(async_main()).unwrap();
}
