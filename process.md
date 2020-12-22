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

