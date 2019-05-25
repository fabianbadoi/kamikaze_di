# Kamikaze DI

This is what a dependency injection container for Rust. It's inspired by container libraries in other languages.

I mostly want to know what people think, and if anyone would want to use something like this.

## What it looks like in action
```rust
let config = container.resolve::<Config>(); // simple resolve via the OmniResolver trait
let config: Config = cotantiner.resolve();  // using AutoResolver and Resolve/ResolveToRc

// deriving the Resolve trait teaches the container how to create it
#[derive(Resolve, Clone)]
struct DabatabaseConnection {
    ...
}
```

## Discussion

There are two important concepts in Rust: ownershipt and mutability. Both influence the design of our DI container.

### Ownership

Data can only have one owner, so who ownes what when do you do:
```rust
let db = container.resolve::<Database>();
```

The `db` object comes from inside the container. It must have owned it at some point, and now the current scope does.
So what happens when we resolve `Database` again?

#### Factories
One way of going about it is to have the container act as a factory. While that's desired sometimes (and supported
via `.register_factory::<T>()`), it's certainly not a sane default, how would we share things?

#### Copies
If we copy or clone objects before returning them, then we can share things. But there are things that should probably
never be shared.

#### Cloning the unclonable?
Other languages don't have this problem since everything lives in the heap and is reference counted. Sounds like Rc<>,
doesn't it.

#### Using Rc
The type signature of all the register functions on the container builder is something like:
```rust
    fn register<T>(&mut self, item: T) -> Result<()> where T: Clone
```

We always require Clone, some types will be OK with this. For the others, you can use Rc<T>.
```rust
let database = ...;
builder.register(Rc::new(database));
```

Rc can also be used with trait objects:
```rust
let database: MysqlConnection = ...;
builder.register::<Rc<Database>>(Rc::new(database));
```


### What about mutablility?

If you're getting cloned objects, mutability is your responsibility.
If you're using Rc, there's a different story: Rc::get\_mut() will always return None because the container will always
keep a refence to it. You will need to use interior mutability.

## What about Sync
That's a very good question.

Basically, I don't want to add it. I just want to start a discussion, I don't intend to maintain a tool I won't
use myself, and I don't write enough Rust code to do that.


## Auto-derive
If the `AutoResolvable` trait is in scope, the container will try to figure out how to create dependencies itself.
This would usually be done with reflection at runtime, but rust doesn't support that.

Any type implements `Resolve` or `ResolveToRc` can be resolved this way. Of course, writing all that code youself is
tedious. So why not just derive that?

```rust
// Just derive this trait
#[derive(Resolve, Clone)]
struct YourStruct {
// ...
}
```

All of that types dependencies will need to either derive `Resolve`, `ResolveToRc` or be registered with the container.


## Errors
I'll be frank, they suck. The look like:
> Could not resolve TypeId { 1589273952 }

The container uses `std::any::TypeId`, and that's just a u64. It could be possible to do get something better done by
using macros.


## Is if possible to insert wrong type
I don't know, find out is on my TODO list.

## Examples
There are examples in repo and the documentation.
