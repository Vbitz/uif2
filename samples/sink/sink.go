package main

import (
	"log"
	"time"

	"github.com/vbitz/uif2/v2"
)

func main() {
	client, err := uif2.Dial()
	if err != nil {
		log.Fatal(err)
	}

	lbl := uif2.NewLabel("Hello")

	client.Root.Append(lbl)

	txt := uif2.NewTextInput()

	client.Root.Append(txt)

	txt.AddOnChanged(func() {
		lbl.SetText(txt.Text)

		err := client.Flush()
		if err != nil {
			log.Fatal(err)
		}
	})

	err = client.Flush()
	if err != nil {
		log.Fatal(err)
	}

	for {
		time.Sleep(1 * time.Second)
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
