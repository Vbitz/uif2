package main

import (
	"log"

	"github.com/vbitz/uif2/v2"
)

func main() {
	client, err := uif2.Dial()
	client.Root.Append(uif2.NewLabel("Hello"))
	err = client.Flush()
	if err != nil {
		log.Fatal(err)
	}

	// conn.WriteJSON(uif2.Transaction{
	// 	ClientId: "gouif2",
	// 	Edits: []uif2.EditCommand{
	// 		{
	// 			AppendChild: &uif2.CmdAppendChild{
	// 				ParentId: uif2.ROOT,
	// 				ObjectId: 10,
	// 				Node: uif2.Node{Label: &uif2.Label{
	// 					Text: "Hello, World",
	// 				}},
	// 			},
	// 		},
	// 	},
	// })
}
