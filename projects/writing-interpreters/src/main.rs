use writing_interpreters::{
    interpreter::{CellPtr, Mutator, MutatorView, RuntimeError, ScopedPtr, TypeList},
    memory::AllocObject,
};

struct Stack {}

impl AllocObject<TypeList> for Stack {
    const TYPE_ID: TypeList = TypeList::Text;
}

struct Roots {
    stack: CellPtr<Stack>,
}

impl Roots {
    fn new(stack: ScopedPtr<'_, Stack>) -> Roots {
        Roots {
            stack: CellPtr::new_with(stack),
        }
    }
}

struct Interpreter {}

impl Mutator for Interpreter {
    type Input = ();
    type Output = Roots;

    fn run(&self, mem: &MutatorView, input: Self::Input) -> Result<Self::Output, RuntimeError> {
        let stack = mem.alloc(Stack {})?; // returns a ScopedPtr<'_, Stack>
        let roots = Roots::new(stack);
        let stack_ptr = roots.stack.get(mem); // returns a ScopedPtr<'_, Stack>

        Ok(roots)
    }
}

fn main() {
    println!("Hello, world!");
}
