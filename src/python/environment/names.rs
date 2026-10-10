use crate::lockfiles::python::normalized;

const DISTRIBUTIONS_OF_MODULES: &[(&str, &str)] = &[
    ("yaml", "pyyaml"),
    ("bs4", "beautifulsoup4"),
    ("dateutil", "python-dateutil"),
    ("PIL", "pillow"),
    ("sklearn", "scikit-learn"),
    ("cv2", "opencv-python"),
    ("jwt", "pyjwt"),
    ("dotenv", "python-dotenv"),
    ("magic", "python-magic"),
    ("Crypto", "pycryptodome"),
    ("OpenSSL", "pyopenssl"),
    ("attr", "attrs"),
    ("pkg_resources", "setuptools"),
    ("multipart", "python-multipart"),
    ("jose", "python-jose"),
    ("docx", "python-docx"),
    ("pptx", "python-pptx"),
    ("telegram", "python-telegram-bot"),
    ("discord", "discord-py"),
    ("serial", "pyserial"),
    ("usb", "pyusb"),
    ("git", "gitpython"),
    ("github", "pygithub"),
    ("slugify", "python-slugify"),
    ("socks", "pysocks"),
    ("zmq", "pyzmq"),
    ("google", "protobuf"),
    ("fitz", "pymupdf"),
    ("websocket", "websocket-client"),
    ("Levenshtein", "python-levenshtein"),
    ("lzstring", "lz-string"),
    ("mpl_toolkits", "matplotlib"),
    ("win32api", "pywin32"),
    ("ruamel", "ruamel-yaml"),
];

pub fn candidate_distributions(module: &str) -> Vec<String> {
    let base = normalized(module);
    let mut candidates: Vec<String> = DISTRIBUTIONS_OF_MODULES
        .iter()
        .filter(|(known, _)| *known == module)
        .map(|(_, distribution)| distribution.to_string())
        .collect();
    candidates.push(base.clone());
    candidates.push(format!("python-{base}"));
    candidates.push(format!("py{base}"));
    candidates
}
