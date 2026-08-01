use viewkit::prelude::*;

struct HelloWorldApp;

impl App for HelloWorldApp {
    type Body = Box<dyn View + 'static>;

    fn new() -> Self {
        Self
    }

    fn window(&self) -> WindowOptions {
        WindowOptions::new("Hello, world!")
            .size(480.0, 320.0)
            .resizable(true)
    }

    fn body(&self, _context: &ViewContext) -> Box<dyn View + 'static> {
        Box::new(
            VStack::new()
                .alignment(StackAlignment::Center)
                .distribution(StackDistribution::Center)
                .child(
                    Text::new("Hello, world!")
                        .font_size(32.0)
                        .alignment(TextAlignment::Center),
                ),
        )
    }
}

fn main() -> Result<(), ViewKitError> {
    viewkit::run::<HelloWorldApp>()
}