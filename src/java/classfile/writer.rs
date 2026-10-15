use super::CallKind;

const MAGIC: u32 = 0xCAFE_BABE;
const MAJOR_VERSION: u16 = 61;
const PUBLIC_SUPER: u16 = 0x0021;
const PUBLIC: u16 = 0x0001;
const NATIVE: u16 = 0x0100;
const VOID_DESCRIPTOR: &str = "()V";
const REFERENCE_STATIC: u8 = 6;
const SWITCH_LOW: i32 = 0x0102_0304;

pub struct TestCall<'a> {
    pub class: &'a str,
    pub name: &'a str,
    pub kind: CallKind,
}

pub struct TestMethod<'a> {
    pub name: &'a str,
    pub is_native: bool,
    pub calls: &'a [TestCall<'a>],
    pub lambda: Option<TestCall<'a>>,
}

pub struct TestClass<'a> {
    pub name: &'a str,
    pub methods: &'a [TestMethod<'a>],
}

#[derive(Default)]
struct Pool {
    bytes: Vec<u8>,
    count: u16,
}

impl Pool {
    fn add(&mut self, entry: Vec<u8>) -> u16 {
        self.count += 1;
        self.bytes.extend(entry);
        self.count
    }

    fn utf8(&mut self, text: &str) -> u16 {
        let mut entry = vec![1];
        entry.extend((text.len() as u16).to_be_bytes());
        entry.extend(text.as_bytes());
        self.add(entry)
    }

    fn class(&mut self, name: &str) -> u16 {
        let name = self.utf8(name);
        self.add([vec![7], name.to_be_bytes().to_vec()].concat())
    }

    fn method(&mut self, call: &TestCall) -> u16 {
        let class = self.class(call.class);
        let name = self.utf8(call.name);
        let descriptor = self.utf8(VOID_DESCRIPTOR);
        let name_and_type = self.add(
            [
                vec![12],
                name.to_be_bytes().to_vec(),
                descriptor.to_be_bytes().to_vec(),
            ]
            .concat(),
        );
        let tag = if call.kind == CallKind::Interface {
            11
        } else {
            10
        };
        self.add(
            [
                vec![tag],
                class.to_be_bytes().to_vec(),
                name_and_type.to_be_bytes().to_vec(),
            ]
            .concat(),
        )
    }

    fn handle(&mut self, call: &TestCall) -> u16 {
        let reference = self.method(call);
        self.add([vec![15, REFERENCE_STATIC], reference.to_be_bytes().to_vec()].concat())
    }
}

pub fn class_bytes(class: &TestClass) -> Vec<u8> {
    let mut pool = Pool::default();
    let this_class = pool.class(class.name);
    let super_class = pool.class("java/lang/Object");
    let code_name = pool.utf8("Code");
    let bootstrap_name = pool.utf8("BootstrapMethods");
    let mut bootstrap_entries: Vec<u8> = Vec::new();
    let mut bootstrap_count: u16 = 0;
    let mut methods = Vec::new();
    for method in class.methods {
        let name = pool.utf8(method.name);
        let descriptor = pool.utf8(VOID_DESCRIPTOR);
        let mut code = vec![0xaa, 0, 0, 0];
        code.extend(0i32.to_be_bytes());
        code.extend(SWITCH_LOW.to_be_bytes());
        code.extend((SWITCH_LOW + 1).to_be_bytes());
        code.extend([0u8; 8]);
        code.extend([0x10, 5, 0x57]);
        for call in method.calls {
            let index = pool.method(call);
            let opcode = match call.kind {
                CallKind::Virtual => 0xb6,
                CallKind::Special => 0xb7,
                CallKind::Static => 0xb8,
                CallKind::Interface => 0xb9,
            };
            code.push(opcode);
            code.extend(index.to_be_bytes());
            if call.kind == CallKind::Interface {
                code.extend([1, 0]);
            }
        }
        if let Some(lambda) = &method.lambda {
            let factory = pool.handle(&TestCall {
                class: "java/lang/invoke/LambdaMetafactory",
                name: "metafactory",
                kind: CallKind::Static,
            });
            let target = pool.handle(lambda);
            bootstrap_entries.extend(factory.to_be_bytes());
            bootstrap_entries.extend(1u16.to_be_bytes());
            bootstrap_entries.extend(target.to_be_bytes());
            let run = pool.utf8("run");
            let descriptor = pool.utf8(VOID_DESCRIPTOR);
            let name_and_type = pool.add(
                [
                    vec![12],
                    run.to_be_bytes().to_vec(),
                    descriptor.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            let dynamic = pool.add(
                [
                    vec![18],
                    bootstrap_count.to_be_bytes().to_vec(),
                    name_and_type.to_be_bytes().to_vec(),
                ]
                .concat(),
            );
            bootstrap_count += 1;
            code.push(0xba);
            code.extend(dynamic.to_be_bytes());
            code.extend([0, 0]);
        }
        code.push(0xb1);
        let mut entry = Vec::new();
        let flags = if method.is_native {
            PUBLIC | NATIVE
        } else {
            PUBLIC
        };
        entry.extend(flags.to_be_bytes());
        entry.extend(name.to_be_bytes());
        entry.extend(descriptor.to_be_bytes());
        if method.is_native {
            entry.extend(0u16.to_be_bytes());
        } else {
            entry.extend(1u16.to_be_bytes());
            entry.extend(code_name.to_be_bytes());
            entry.extend(((code.len() + 12) as u32).to_be_bytes());
            entry.extend([0, 4, 0, 4]);
            entry.extend((code.len() as u32).to_be_bytes());
            entry.extend(&code);
            entry.extend([0, 0, 0, 0]);
        }
        methods.push(entry);
    }
    let mut bytes = Vec::new();
    bytes.extend(MAGIC.to_be_bytes());
    bytes.extend(0u16.to_be_bytes());
    bytes.extend(MAJOR_VERSION.to_be_bytes());
    bytes.extend((pool.count + 1).to_be_bytes());
    bytes.extend(&pool.bytes);
    bytes.extend(PUBLIC_SUPER.to_be_bytes());
    bytes.extend(this_class.to_be_bytes());
    bytes.extend(super_class.to_be_bytes());
    bytes.extend([0, 0, 0, 0]);
    bytes.extend((methods.len() as u16).to_be_bytes());
    for method in methods {
        bytes.extend(method);
    }
    bytes.extend(1u16.to_be_bytes());
    bytes.extend(bootstrap_name.to_be_bytes());
    bytes.extend(((bootstrap_entries.len() + 2) as u32).to_be_bytes());
    bytes.extend(bootstrap_count.to_be_bytes());
    bytes.extend(bootstrap_entries);
    bytes
}
