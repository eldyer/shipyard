# Running the World with Systems

There are two ways to operate on the world.

## Systems

Systems are created by implementing the `System` trait. It's a bit involved- but don't worry, there's a helper macro we'll get to down below that makes this much simpler!

First, implementing the trait directly. It looks like:

1. Define a system
```rust, noplaypen
struct CreateEmpty;
impl<'a> System<'a> for CreateEmpty {
    type Data = (EntitiesMut, &'a mut Empty);
    
    fn run((entities, empties): <T::Data as SystemData>::View) { ... }
}
```

First we create a struct, it could be an enum, empty or not it doesn't really matter it's just here to attach the impl on. Note that even if you don't make it empty you won't have access to the struct inside the system.

We then choose which storages we want with the `Data` associated type. We'll see in the next chapters what types can be used in detailed.

Finally, we get a tuple as `run` argument containing views to the storages we asked with `Data`. You can use `<T::Data as SystemData>::View` as type, it's a bit esoteric but will work with whatever you borrow. Or you could specify the exact types.

2. Add the workload
```rust, noplaypen
world.add_workload("Creators", CreateEmpty);
```

3. Run the workload (which will run its systems)
```rust, noplaypen
world.run_workload("Creators");
```

Adding multiple systems to a workload is only a matter of expanding the second argument to a tuple. For example: 

1. Define another system - this time using the macro
```rust, noplaypen
#[system(CreateCount)]
fn run (entities: &mut Entities, counts: &mut Count) { ... }
```

It will also take care of transforming its arguments into tuple(s), no need to worry about it anymore.

2. Add the workload
```rust, noplaypen
world.add_workload("Creators", (CreateEmpty, CreateCount));
```

3. Run the workload
```rust, noplaypen
world.run_workload("Creators");
```

This is great because it avoids a ton of boilerplate and provides [outer-parallelism](../concepts/inner-outer-parallelism.md) without having to do anything.

There's a few points to keep in mind about workloads:
1. Workloads will run its systems in parallel where possible. If they can't be run in parallel, then systems run sequentially left-to-right.
2. A workload cannot be modified once its defined. Think of it more as a one-time-setup thing than something you do dynamically at runtime. Workloads are cheap so even if you make a few with similar systems it's ok.
3. Adding a workload with a name that already exists will replace it.

## Running directly

Sometimes a world needs to be run where it has access to variables that are not registered in the ECS.

For this use case, `run` may be called:

```rust, noplaypen
world.run::<(EntitiesMut, &mut Empty), _, _>(|(entities, empties)| { ... });
```

We use turbofish to specify what storages we want to borrow just like in systems. We then have two underscores, that's because the methods takes two additional generics but the compiler will almost always know which types you are referring to.

The first underscore correspond to the type returned by `run` for example:

```rust, noplaypen
let entity = world.run::<(EntitiesMut, &mut Empty), _, _>(|(entities, empties)| { ... });
```

The last underscore is the complete type of the closure.
