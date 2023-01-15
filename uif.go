package uif2

import (
	"fmt"
	"log"

	"github.com/gorilla/websocket"
)

const ROOT = 0xff_ff_ff_ff

var currentId uint32 = 1

func newId() uint32 {
	currentId += 1
	return currentId
}

var currentUid uint32 = 1

func newUid() string {
	currentUid += 1
	return fmt.Sprintf("%d", currentUid)
}

type Client struct {
	conn      *websocket.Conn
	currentTx *Transaction
	Root      *Node
	callbacks []func(*Event)
}

func (c *Client) addEventCallback(cb func(*Event)) {
	c.callbacks = append(c.callbacks, cb)
}

func (c *Client) eventLoop() {
	for {
		var ev Event

		err := c.conn.ReadJSON(&ev)
		if err != nil {
			log.Printf("error reading event: %v", err)
			return
		}

		for _, cb := range c.callbacks {
			cb(&ev)
		}
	}
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

	client := &Client{
		conn: conn,
		currentTx: &Transaction{
			ClientId: "gouif2",
			Edits:    []EditCommand{},
		},
	}

	go client.eventLoop()

	client.Root = &Node{
		state: nodeState{
			client: client,
			id:     ROOT,
		},
	}

	return client, nil
}

type nodeState struct {
	client   *Client
	id       uint32
	children []NodeFuncs
}

type Label struct {
	node *Node

	Text string `json:"text"`
}

// implements NodeFuncs
func (n *Label) Append(node NodeFuncs)    { n.node.Append(node) }
func (n *Label) SetClient(client *Client) { n.node.SetClient(client) }
func (n *Label) Node() *Node              { return n.node }

func (n *Label) SetText(t string) {
	n.Text = t
	n.node.Replace()
}

func NewLabel(text string) *Label {
	lbl := &Label{
		node: newNode(),
		Text: text,
	}
	lbl.node.Label = lbl
	return lbl
}

type TextInput struct {
	node *Node

	Text      string `json:"text"`
	OnChanged string `json:"on_changed"`
}

func NewTextInput() *TextInput {
	txt := &TextInput{
		node:      newNode(),
		Text:      "",
		OnChanged: newUid(),
	}
	txt.node.TextInput = txt
	return txt
}

func (n *TextInput) AddOnChanged(cb func()) {
	n.node.AddUpdateListener(n.OnChanged, cb)
}

// implements NodeFuncs
func (n *TextInput) Append(node NodeFuncs)    { n.node.Append(node) }
func (n *TextInput) SetClient(client *Client) { n.node.SetClient(client) }
func (n *TextInput) Node() *Node              { return n.node }

type ComboBox struct {
	node *Node

	Label     string   `json:"label"`
	Selected  string   `json:"selected"`
	Options   []string `json:"options"`
	OnChanged string   `json:"on_changed"`
}

// implements NodeFuncs
func (n *ComboBox) Append(node NodeFuncs)    { n.node.Append(node) }
func (n *ComboBox) SetClient(client *Client) { n.node.SetClient(client) }
func (n *ComboBox) Node() *Node              { return n.node }

var (
	_ NodeFuncs = &Node{}
	_ NodeFuncs = &Label{}
	_ NodeFuncs = &TextInput{}
	_ NodeFuncs = &ComboBox{}
)

type NodeFuncs interface {
	SetClient(client *Client)
	Append(node NodeFuncs)
	Node() *Node
}

type Node struct {
	state nodeState

	Label     *Label     `json:",omitempty"`
	TextInput *TextInput `json:",omitempty"`
	ComboBox  *ComboBox  `json:",omitempty"`
}

func (n *Node) update(node Node) {
	if node.ComboBox != nil {
		n.ComboBox.Selected = node.ComboBox.Selected
	} else if node.TextInput != nil {
		n.TextInput.Text = node.TextInput.Text
	}
}

func (n *Node) Node() *Node { return n }

func (n *Node) SetClient(client *Client) {
	n.state.client = client
}

func (n *Node) Append(node NodeFuncs) {
	n.state.children = append(n.state.children, node)

	if n.state.client != nil {
		n.state.client.currentTx.append(n, node)

		node.SetClient(n.state.client)
	}
}

func (n *Node) Replace() {
	if n.state.client != nil {
		n.state.client.currentTx.replace(n)
	}
}

func (n *Node) AddUpdateListener(evId string, cb func()) {
	if n.state.client != nil {
		n.state.client.addEventCallback(func(ev *Event) {
			if ev.Update != nil {
				n.update(ev.Update.Node)
				cb()
			}
		})
	}
}

func newNode() *Node {
	return &Node{
		state: nodeState{
			id: newId(),
		},
	}
}

type CmdAppendChild struct {
	ParentId uint32 `json:"parent_id"`
	ObjectId uint32 `json:"object_id"`
	Node     *Node  `json:"node"`
}

type CmdReplaceNode struct {
	ObjectId uint32 `json:"object_id"`
	Node     *Node  `json:"node"`
}

type EditCommand struct {
	AppendChild *CmdAppendChild `json:",omitempty"`
	ReplaceNode *CmdReplaceNode `json:",omitempty"`
}

type EvUpdate struct {
	Id       string `json:"id"`
	ObjectId uint32 `json:"object_id"`
	Node     Node   `json:"node"`
}

type EvClicked struct {
	Id string `json:"id"`
}

type Event struct {
	Update  *EvUpdate  `json:",omitempty"`
	Clicked *EvClicked `json:",omitempty"`
}

type Transaction struct {
	ClientId string        `json:"client_id"`
	Edits    []EditCommand `json:"edits"`
}

func (t *Transaction) append(parent *Node, node NodeFuncs) {
	t.Edits = append(t.Edits, EditCommand{AppendChild: &CmdAppendChild{
		ParentId: parent.state.id,
		ObjectId: node.Node().state.id,
		Node:     node.Node(),
	}})
}

func (t *Transaction) replace(node *Node) {
	t.Edits = append(t.Edits, EditCommand{ReplaceNode: &CmdReplaceNode{
		ObjectId: node.state.id,
		Node:     node,
	}})
}
