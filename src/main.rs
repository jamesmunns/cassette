trait TopQueueItem {
    type Context;
    type Substate: BottomQueueItem<Context = Self::Context>;

    fn poll(&self, ctxt: &mut Self::Context) -> Result<bool, ()>;
    fn substates() -> Vec<Self::Substate>;
}

trait BottomQueueItem {
    type Context;

    fn poll(&self, ctxt: &mut Self::Context) -> Result<bool, ()>;
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

enum DemoTopStates {
    ZeroToTen,
    FiveToZero
}

impl TopQueueItem for DemoTopStates {
    type Context = Demo;

    fn poll(&self, ctxt: &mut Self::Context) -> Result<bool, ()> {
        (match self {
            DemoTopStates::ZeroToTen => Demo::zero_to,
            DemoTopStates::FiveToZero => Demo::start_at_five,
        })(ctxt)
    }
}

enum DemoBottomStates {
    StartAtZero,
    StartAtFive,
    AddOneUntilTen,
    SubOneUntilZero,
}

impl BottomQueueItem for DemoBottomStates {
    type Context = Demo;

    fn poll(&self, ctxt: &mut Self::Context) -> Result<bool, ()> {
        (match self {
            DemoBottomStates::StartAtZero => Demo::start_at_zero,
            DemoBottomStates::StartAtFive => Demo::start_at_five,
            DemoBottomStates::AddOneUntilTen => Demo::add_one_until_ten,
            DemoBottomStates::SubOneUntilZero => Demo::sub_one_until_zero,
        })(ctxt)
    }
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

    fn start_at_five(&mut self) -> Result<bool, ()> {
        self.lol = 5;
        Ok(true)
    }

    fn add_one_until_ten(&mut self) -> Result<bool, ()> {
        self.lol += 1;
        Ok(self.lol >= 10)
    }

    fn sub_one_until_zero(&mut self) -> Result<bool, ()> {
        self.lol -= 1;
        Ok(self.lol == 0)
    }
}

fn main() {
    println!("Hello, world!");
}
