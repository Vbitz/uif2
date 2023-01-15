package uif2

import "github.com/gorilla/websocket"

const ROOT = 0xff_ff_ff_ff

var currentId uint32 = 1

func newId() uint32 {
	currentId += 1
	return currentId
}

type Client struct {
	conn      *websocket.Conn
	currentTx *Transaction
	Root      *Node
}

func (c *Client) Flush() error {
	err := c.conn.WriteJSON(c.currentTx)
	if err != nil {
		return err
	}
	c.currentTx = &Transaction{
		ClientId: "gouif2",
		Edits:    []EditCommand{},
	}
	return nil
}

func Dial() (*Client, error) {
	conn, _, err := websocket.DefaultDialer.Dial("ws://127.0.0.1:3012/", nil)
	if err != nil {
		return nil, err
	}

	return &Client{
		conn: conn,
		currentTx: &Transaction{
			ClientId: "gouif2",
			Edits:    []EditCommand{},
		},
	}, nil
}

type nodeState struct {
	client   *Client
	id       uint32
	children []Node
}

type Label struct {
	node *Node

	Text string `json:"text"`
}

func (n *Label) SetText(t string) {
	n.Text = t
}

func NewLabel(text string) *Label {
	return &Label{
		node: newNode(),
		Text: text,
	}
}

type TextInput struct {
	node *Node

	Text      string `json:"text"`
	OnChanged string `json:"on_changed"`
}

type ComboBox struct {
	node *Node

	Label     string   `json:"label"`
	Selected  string   `json:"selected"`
	Options   []string `json:"options"`
	OnChanged string   `json:"on_changed"`
}

var ()

type NodeFuncs interface {
	Append(node NodeFuncs)
}

type Node struct {
	state *nodeState

	Label     *Label     `json:",omitempty"`
	TextInput *TextInput `json:",omitempty"`
	ComboBox  *ComboBox  `json:",omitempty"`
}

func (n *Node) Append(node NodeFuncs) {

}

func newNode() *Node {
	return &Node{
		state: &nodeState{
			id: newId(),
		},
	}
}

type CmdAppendChild struct {
	ParentId uint32 `json:"parent_id"`
	ObjectId uint32 `json:"object_id"`
	Node     Node   `json:"node"`
}

type CmdReplaceNode struct {
	ObjectId uint32 `json:"object_id"`
	Node     Node   `json:"node"`
}

type EditCommand struct {
	AppendChild *CmdAppendChild `json:",omitempty"`
	ReplaceNode *CmdReplaceNode `json:",omitempty"`
}

type Transaction struct {
	ClientId string        `json:"client_id"`
	Edits    []EditCommand `json:"edits"`
}
