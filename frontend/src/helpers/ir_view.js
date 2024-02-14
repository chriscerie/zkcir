function createTreeNode(name, children = []) {
  return { name, children };
}

function formatBinaryOperator(node) {
  const { lhs, binop, rhs } = node;

  // Process lhs and rhs nodes
  const lhsNode = processNode(lhs);
  const rhsNode = processNode(rhs);
  const nodeName = `${binop}()`;

  return createTreeNode(nodeName, [rhsNode, lhsNode].filter(Boolean).reverse());
}

function processNode(node) {
  if (!node) {
    return null;
  }

  const nodeType = Object.keys(node)[0];
  const value = node[nodeType];

  switch (nodeType) {
    case 'Local':
      return createTreeNode(
        nodeType,
        [processNode(value[0]), processNode(value[1])]
          .filter(Boolean)
          .reverse(),
      );
    case 'BinaryOperator':
      return formatBinaryOperator(value);
    case 'Ident':
      switch (Object.keys(value)[0]) {
        case 'String':
          return createTreeNode(value.String);
        case 'Wire':
          return createTreeNode(
            `wire!(${value.Wire.row}, ${value.Wire.column})`,
          );
        case 'VirtualWire':
          return createTreeNode(`v_wire!(${value.VirtualWire.index})`);
        default:
          return createTreeNode(Object.keys(value)[0]);
      }
    case 'Verify': {
      const childNode = processNode(value);
      return createTreeNode('verify!()', childNode ? [childNode] : []);
    }
    case 'Value':
      return createTreeNode(
        `${Object.keys(value)[0]}(${value[Object.keys(value)[0]]})`,
      );
    default:
      if (typeof value === 'object') {
        return createTreeNode(
          nodeType,
          [processNode(value)].filter(Boolean).reverse(),
        );
      } else if (typeof value === 'string') {
        return createTreeNode(value);
      }

      return null;
  }
}

const ir_view = {
  generateTree: (ir) => {
    const rootNode = createTreeNode('Circuit');

    rootNode.children = ir.stmts
      .map((expr) => processNode(expr))
      .filter(Boolean)
      .reverse();

    console.log(JSON.stringify(rootNode));
    return rootNode;
  },
};

export default ir_view;
