import {ObjectId} from "mongodb";

const MongoClient = require('mongodb').MongoClient;

interface Cell {
    pos: number;
    color: string;
    is_king: boolean;
    stricken: boolean;
}

interface Node {
    cells: (Cell | null)[];
    next_move: string;
    W: number;
    N: number;
    NN: number | null;
}

interface Item {
    node: Node;
    child: Node;
}

interface WrapItem {
    _id: ObjectId
    item: Item;
    repetitions: number;
}

interface NewItem {
    v_node: number[];
    v_child: number[];
    quality: {
        node: {
            W: number;
            N: number;
            NN: number | null;
        };
        child: {
            W: number;
            N: number;
            NN: number | null;
        };
    };
}

interface NewWrapItem {
    _id: ObjectId
    item: NewItem;
    repetitions: number;
}


async function updateItems() {
    const uri = 'mongodb://localhost:27017';
    const client = new MongoClient(uri);


    await client.connect();
    const database = client.db('checkers');
    const collection = database.collection('nodes');

    // Получение всех элементов старой коллекции
    const cursor  = await collection.find();

    while (await cursor.hasNext()) {
        const oldItem: WrapItem = await cursor.next();
        {
            const itemId = oldItem._id;
            try {
                const newItem: NewWrapItem = {
                    _id: oldItem._id,
                    item: {
                        v_child: [...oldItem.item.child.cells.map(x =>
                            (!x) ? 0 :
                                (x.is_king ? 3 : 1) * (x.color == "White" ? 1 : -1)), oldItem.item.child.next_move == "White" ? 1 : -1],
                        v_node: [...oldItem.item.node.cells.map(x =>
                            (!x) ? 0 :
                                (x.is_king ? 3 : 1) * (x.color == "White" ? 1 : -1)), oldItem.item.child.next_move == "White" ? 1 : -1],
                        quality: {
                            child: {N: oldItem.item.child.N, W: oldItem.item.child.W, NN: oldItem.item.child.NN},

                            node: {N: oldItem.item.node.N, W: oldItem.item.node.W, NN: null}
                        }
                    }, repetitions: oldItem.repetitions
                };

                // Запись нового элемента с тем же _id
                const updateQuery = {_id: itemId};
                const updateDoc = {$set: newItem};
                await collection.updateOne(updateQuery, updateDoc);
            } catch (err) {
            }

        }
    }
    // Обновление каждого элемента

    console.log("finished")

}


updateItems().then(r => () => {
    console.log("end")
});

// async function updateDocuments() {
//     const uri = "mongodb://localhost:27017";
//     const client = new MongoClient(uri);
//
//     let n =0
//     try {
//         await client.connect();
//         const database = client.db("checkers");
//         const collection = database.collection("nodes");
//
//         // Находим документы, которые нужно обновить
//         const documentsToUpdate = await collection.find({}).toArray();
//
//         // Обновляем каждый документ
//         for (const document of documentsToUpdate) {
//             const itemId = document._id;
//             const item = document.item;
//
//             try {// Извлекаем значения полей из поля "item" и обновляем документ
//                 const v_node = item.item.v_node;
//                 const v_child = item.item.v_child;
//                 const qualityNode = item.item.quality.node;
//                 const qualityChild = item.item.quality.child;
//                 await collection.updateOne(
//                     { _id: itemId },
//                     {
//                         $set: {
//                             "item.v_node": v_node,
//                             "item.v_child": v_child,
//                             "item.quality.node.W": qualityNode.W ,
//                             "item.quality.node.N": qualityNode.N ,
//                             "item.quality.node.NN": null,
//                             "item.quality.child.W": qualityChild.W,
//                             "item.quality.child.N": qualityChild.N,
//                             "item.quality.child.NN": qualityChild.NN,
//                         }
//                     }
//                 );
//                 n++
//
//             } catch (err) {
//                 await collection.deleteOne({ _id: itemId })
//                 continue
//             }
//         }
//
//         console.log("Документы успешно обновлены", n);
//     } catch (error) {
//         console.error("Произошла ошибка при обновлении документов:", error);
//     } finally {
//         await client.close();
//     }
// }
//
// updateDocuments();

