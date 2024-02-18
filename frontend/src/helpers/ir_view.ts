interface IRData {
  config: Config;
  stmts: Stmt[];
  public_wire_inputs: PublicWireInput[];
  public_virtual_wire_inputs: VirtualWireInput[];
}

interface Config {
  num_wires: number;
}

interface PublicWireInput {
  // Assuming the structure, since it wasn't detailed in the provided data
}

interface VirtualWireInput {
  index: number;
  value: NumericValue;
}

interface NumericValue {
  U64: number;
}

type Stmt =
  | { Local: [LocalNode, LocalNode] } // Adjusted based on your data, assuming always two elements
  | { Verify: VerifyNode };

interface LocalNode {
  String?: string;
  Ident?: IdentNode;
}

interface IdentNode {
  VirtualWire?: VirtualWire;
  String?: string;
}

interface VirtualWire {
  index: number;
  value: NumericValue;
}

interface VerifyNode {
  BinaryOperator: BinaryOperatorNode;
}

interface BinaryOperatorNode {
  lhs: OperandNode;
  binop: string; // Enumerate as 'LessThan' | 'Exponent' | ... for more precise typing
  rhs: OperandNode;
}

type OperandNode =
  | { Ident: IdentNode }
  | { Value: NumericValue }
  | { BinaryOperator: BinaryOperatorNode };

interface TreeNode {
  name: string;
  children?: TreeNode[];
  collapsed?: boolean;
}

function createTreeNode(name: string, children: TreeNode[] = []): TreeNode {
  return { name, children };
}

function formatBinaryOperator(node: BinaryOperatorNode): TreeNode {
  const lhsNode = processOperandNode(node.lhs);
  const rhsNode = processOperandNode(node.rhs);
  const nodeName = `${node.binop}()`;
  // Apply a type assertion here
  return createTreeNode(nodeName, [lhsNode, rhsNode].filter(Boolean) as TreeNode[]);
}

function processOperandNode(node: OperandNode): TreeNode | null {
  if ('Ident' in node) {
    return processIdentNode(node.Ident);
  } else if ('Value' in node) {
    return createTreeNode(`Value(U64: ${node.Value.U64})`);
  } else if ('BinaryOperator' in node) {
    return formatBinaryOperator(node.BinaryOperator);
  }
  return null;
}

function processIdentNode(node: IdentNode): TreeNode {
  if (node.VirtualWire) {
    return createTreeNode(`VW(index: ${node.VirtualWire.index}, value: U64: ${node.VirtualWire.value.U64})`);
  } else if (node.String) {
    return createTreeNode(node.String);
  }
  return createTreeNode('Unknown Ident');
}

function processNode(node: Stmt): TreeNode | null {
  if ('Local' in node) {
    const [first, second] = node.Local;
    // Apply a type assertion here
    return createTreeNode('Local', [processLocalNode(first), processLocalNode(second)].filter(Boolean) as TreeNode[]);
  } else if ('Verify' in node) {
    return createTreeNode('Verify', [formatBinaryOperator(node.Verify.BinaryOperator)]);
  }
  return null; // Fallback for unknown types
}

function processLocalNode(local: LocalNode): TreeNode | null {
  if (local.String) {
    return createTreeNode(local.String);
  } else if (local.Ident) {
    return processIdentNode(local.Ident);
  }
  return null;
}

const ir_view = {
  generateTree: (irString: string): TreeNode => {
    const ir: IRData = JSON.parse(irString);
    const rootNode = createTreeNode('Circuit');
    rootNode.children = ir.stmts.map(processNode).filter(Boolean) as TreeNode[];
    return rootNode;
  },
};

export default ir_view;
