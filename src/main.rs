trait TopQueueItem {
    type Context;
    type Substate: BottomQueueItem<Context = Self::Context>;

    fn poll(ctxt: &mut Self::Context) -> Result<bool, ()>;
    fn substates() -> Vec<Self::Substate>;
}

trait BottomQueueItem {
    type Context;

    fn poll(ctxt: &mut Self::Context) -> Result<bool, ()>;
}

// Queue should be a queue of "top level" state flows
// Each item should be able to produce a subqueue flow on init
    // Is this "on entry"?
    // If so, do we need an "on exit"?
struct Cassette<T, TOP, BTM>
where
    TOP: TopQueueItem<Context = T, Substate = BTM>,
    BTM: BottomQueueItem<Context = T>
{
    queue: Vec<TOP>,
    subqueue: Vec<BTM>,

    // this is the payload
    context: T,
}

struct Demo {
    lol: u32,
}

// Hmmm...
// impl BottomQueueItem for Demo::start_at_zero {
//     type Context = Demo;

//     fn poll(ctxt: &mut Self::Context) -> Result<bool, ()> {
//         todo!()
//     }
// }

impl Demo {
    fn start_at_zero(&mut self) -> Result<bool, ()> {
        self.lol = 0;
        Ok(true)
    }

    fn add_one_until_ten(&mut self) -> Result<bool, ()> {
        self.lol += 1;
        Ok(self.lol >= 10)
    }

    fn sub_one(&mut self) -> Result<bool, ()> {
        self.lol -= 1;
        Ok(true)
    }
}

fn main() {
    println!("Hello, world!");
}
