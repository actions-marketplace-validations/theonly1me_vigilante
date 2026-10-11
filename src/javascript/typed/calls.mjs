import { readFileSync, existsSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";

const input = JSON.parse(readFileSync(0, "utf8"));
const ts = createRequire(input.typescript)(input.typescript);
const libraryFolder = path.dirname(input.typescript);
const files = new Map(input.files.map((file) => [file.name, file]));
const options = {
  allowJs: true,
  checkJs: false,
  noEmit: true,
  skipLibCheck: true,
  allowArbitraryExtensions: true,
  resolveJsonModule: true,
  jsx: ts.JsxEmit.Preserve,
  target: ts.ScriptTarget.ESNext,
  module: ts.ModuleKind.ESNext,
  moduleResolution: ts.ModuleResolutionKind.Bundler,
  types: [],
  maxNodeModuleJsDepth: 1,
};

const isLibrary = (fileName) => fileName.startsWith(libraryFolder + path.sep);
const readText = (fileName) => {
  if (files.has(fileName)) return files.get(fileName).text;
  if (isLibrary(fileName) && existsSync(fileName)) return readFileSync(fileName, "utf8");
  return undefined;
};

const host = {
  getSourceFile: (fileName, languageVersion) => {
    const text = readText(fileName);
    return text === undefined ? undefined : ts.createSourceFile(fileName, text, languageVersion, true);
  },
  getDefaultLibFileName: (compilerOptions) => path.join(libraryFolder, ts.getDefaultLibFileName(compilerOptions)),
  writeFile: () => {},
  getCurrentDirectory: () => "/",
  getCanonicalFileName: (fileName) => fileName,
  useCaseSensitiveFileNames: () => true,
  getNewLine: () => "\n",
  fileExists: (fileName) => readText(fileName) !== undefined,
  readFile: readText,
  resolveModuleNameLiterals: (literals, containingFile) => {
    const imports = files.get(containingFile)?.imports ?? {};
    return literals.map((literal) => {
      const target = imports[literal.text];
      return target === undefined
        ? { resolvedModule: undefined }
        : { resolvedModule: { resolvedFileName: target, extension: ts.extensionFromPath(target), isExternalLibraryImport: !files.get(target)?.own } };
    });
  },
};

const own = input.files.filter((file) => file.own).map((file) => file.name);
const ownFiles = new Set(own);
const program = ts.createProgram(own, options, host);
const checker = program.getTypeChecker();
const lineOf = (sourceFile, position) => sourceFile.getLineAndCharacterOfPosition(position).line + 1;
const unquoted = (text) => text.replace(/^["'`#]|["'`]$/g, "");

function nameOf(declaration) {
  if (declaration.name && !ts.isComputedPropertyName(declaration.name)) return unquoted(declaration.name.getText());
  if (ts.isConstructorDeclaration(declaration)) return "constructor";
  const parent = declaration.parent;
  if (ts.isVariableDeclaration(parent) && ts.isIdentifier(parent.name)) return parent.name.text;
  if (ts.isPropertyAssignment(parent) || ts.isPropertyDeclaration(parent)) return unquoted(parent.name.getText());
  if (ts.isBinaryExpression(parent) && ts.isPropertyAccessExpression(parent.left)) return parent.left.name.text;
  if (ts.isBinaryExpression(parent) && ts.isIdentifier(parent.left)) return parent.left.text;
  if (ts.isExportAssignment(parent)) return "default";
  return undefined;
}

function calleeOf(expression) {
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text;
  if (ts.isIdentifier(expression)) return expression.text;
  return undefined;
}

const hasBody = (declaration) =>
  ts.isFunctionDeclaration(declaration) || ts.isMethodDeclaration(declaration) || ts.isConstructorDeclaration(declaration)
    ? declaration.body !== undefined
    : ts.isFunctionExpression(declaration) || ts.isArrowFunction(declaration);

function typedTarget(node) {
  const declaration = checker.getResolvedSignature(node)?.getDeclaration();
  if (!declaration) return undefined;
  const fileName = declaration.getSourceFile().fileName;
  if (ts.isFunctionTypeNode(declaration) || ts.isCallSignatureDeclaration(declaration)) {
    return files.has(fileName) ? { unanalyzed: true } : undefined;
  }
  if (!files.has(fileName) || !hasBody(declaration)) return undefined;
  const name = nameOf(declaration);
  if (name === undefined) return undefined;
  return { targets: [{ file: fileName, name, line: lineOf(declaration.getSourceFile(), declaration.getStart()) }] };
}

const lines = [];
for (const sourceFile of program.getSourceFiles()) {
  if (!ownFiles.has(sourceFile.fileName)) continue;
  const visit = (node) => {
    if (ts.isCallExpression(node) || ts.isNewExpression(node)) {
      const callee = calleeOf(node.expression);
      const typed = callee === undefined ? undefined : typedTarget(node);
      if (typed !== undefined) {
        lines.push(JSON.stringify({ file: sourceFile.fileName, line: lineOf(sourceFile, node.expression.getStart()), callee, ...typed }));
      }
    }
    ts.forEachChild(node, visit);
  };
  visit(sourceFile);
}
process.stdout.write(lines.join("\n") + "\n");
