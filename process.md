## process

First create your app state which will contain the a vector of paths pointing to your images. For simplicity, I hard code a directory I know have only images within it. Then you have another variable that keeps track of which image you're pointing to within the directory. This is initialized to 0 when app state is created.

```rust
    #[derive(Clone, Data, Lens)]
    struct AppState {
        images: Rc<Vec<PathBuf>>,
        current_image: usize,
    }

    const IMAGE_FOLDER: &str = "./images";
```

Now to create a simple image viewer, we only need two buttons. We will have a left and right button that moves left and right through the vector of paths. Every time we click one of the buttons we change the `AppState` by changing the `current_image` field. This gets updated to point to either the previous image or next image. I added logic to wrap around whenever you reach either end of the vector. So within the code to create the root element, I start with 

Write the coming code into `fn main()`
```rust
    let root = || {
        let left_button = Button::new("left").on_click(|_ctx, data: &mut AppState, _env| {
            if data.current_image == 0 {
                data.current_image = data.images.len() - 1;
            } else {
                data.current_image -= 1;
            }
            dbg!(data.current_image);
        });
        let right_button = Button::new("right").on_click(|_ctx, data: &mut AppState, _env| {
            if data.current_image == data.images.len() - 1 {
                data.current_image = 0
            } else {
                data.current_image += 1;
            }
            dbg!(data.current_image);
        });
    };
```
Now we'll get these buttons displayed. To see progress so far. Since we're going for an image viewer the left and right buttons will be on the outside of the image like `left_button <image> right_button`. So we'll want to put them into a `Flexbox row`. So we'll go ahead and create a flex row layout with:

```rust
    // previous code

    let layout = Flex::row()
        .with_child(left_button)
        .with_child(right_button);
```
I also want to make the background white so I'll wrap layout in a `Container` widget like so:

```rust
    // previous code

    Container::new(layout).background(druid::Color::rgb8(255, 255, 255))
```
We'll be returning this widget from the closure so don't put a semicolon at the end of the statement.

Now to start up the application write this
```rust
let window = WindowDesc::new(root);

// this will read the directory for your hardcoded image folder
// it will iterate and collect them into a vector of PathBufs
// I defined the IMAGE_FOLDER as a constant outside of main right
// below AppState
let paths: Vec<PathBuf> = std::fs::read_dir(IMAGE_FOLDER)
    .unwrap()
    .map(|path| path.unwrap().path())
    .collect();

// launch the application with the AppState
// current_image holds the current position viewing
// within images. You modify this to change your position
// within images and thus changing which image is viewed
AppLauncher::with_window(window)
    .use_simple_logger()
    .launch(AppState {
        images: Rc::new(paths),
        current_image: 0,
    })
    .unwrap();
```

Now you can do `cargo run`. You should get two buttons next to each other. When you click them it should print the new `current_image`. If it doesn't go back to the `on_click` closures and print `current_image` to see how they change. With this, now you can navigate through images. Now we'll have to figure out how display the image between the two buttons.

Now we want to get into the most important part of the project; which is displaying the images. 

We'll have to create a custom widget, but it won't be too complicated. We'll call the widget `DisplayImage` and it will store the `Image` widget.
```rust
#[derive(Clone, Data, Lens)]
pub struct DisplayImage {
    pub image: Rc<Image>,
}
```

Now we'll implement `Widget` for `DisplayImage` like so:

```rust
impl Widget<AppState> for DisplayImage {
    // we won't be responding to any events
    // Display image will only display the image
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {}

    // same thing here, DisplayImage won't be dealing with 
    // lifecycle details
    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {}

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        // handles the case where the directory might be empty
        // nothing should happen
        // you should be using a directory with only images anyways :)
        if data.images.is_empty() {
            return;
        }

        // compare the index of the current image with the index of the
        // old image 
        // if it is different then read in the new image and replace
        // self.image with the new image
        if data.current_image != old_data.current_image {
            let image = image::io::Reader::open(&data.images[data.current_image])
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();
            let (width, height) = image.dimensions();
            let image = ImageBuf::from_raw(
                image.into_raw(),
                ImageFormat::Rgb,
                width as usize,
                height as usize,
            );
            let image = Image::new(image).interpolation_mode(InterpolationMode::Bilinear);
            self.image = Rc::new(image);
            // request a paint here because the image has changed and you
            // want Druid to draw the new image
            // I think i might also want to call ctx.request_layout()?
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> druid::Size {
        // here we defer layouting to the underlying Image widget
        // This makes our life easier, since we won't have to implement
        // a custom image widget
        Rc::get_mut(&mut self.image)
            .unwrap()
            .layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &Env) {
        // again defer painting to the underlying image
        Rc::get_mut(&mut self.image).unwrap().paint(ctx, data, env);
    }
}
```

With this `DisplayImage` widget we can use this and put it between our `left` and `right` buttons. We want to first read in the first image in the folder.

```rust
    // read in all the paths found in the folder

    // this way you'll have access to the first image in the closure
    // that builds the root UI
    let first_image = image::io::Reader::open(&paths[0])
        .unwrap()
        .decode()
        .unwrap()
        .into_rgb8();
    let (width, height) = first_image.dimensions();
    let image = Rc::new(first_image.into_vec());
    let image = first_image.clone();

    // root UI builder closure
```

```rust
    // this code goes above the layout builder

    // build the image by turning the image into an ImageBuf
    let image = ImageBuf::from_raw(
        first_image.to_vec(),
        ImageFormat::Rgb,
        width as usize,
        height as usize,
    );

    // create the image widget from the ImageBuf
    // interpolation mode tells Druid how what algorithm to use when
    // resizing the image
    let image = Image::new(image)
        .interpolation_mode(InterpolationMode::Bilinear)
        .fill_mode(FillStrat::Contain);

    // now wrap image widget in our DisplayImage widget with Rc
    let image = DisplayImage {
        image: Rc::new(image),
    };

```

Now we want to modify the layout builder to include our new widget and add some more parameters for the widget's display.

```rust
    let layout = Flex::row()
        // tells Druid that we want this to fill its parent
        // the parent is the window basically, so this will take up all the
        // the space in the window
        .must_fill_main_axis(true)
        .with_child(left_button)
        // makes the image a flex child so it will take up a large amount
        // of space, the flex params tell it how much larger it should be
        // relative to its siblings
        .with_flex_child(image, FlexParams::new(1.0, None))
        .with_child(right_button)
        // centers the child widgets vertically
        .cross_axis_alignment(CrossAxisAlignment::Center)
        // puts space between each child
        // this essentially centers the image, and moves the left and right
        // buttons to their respective sides of the window
        .main_axis_alignment(MainAxisAlignment::SpaceBetween);
```

