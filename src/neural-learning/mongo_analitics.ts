import {MongoClient, Db, Collection, ObjectId} from 'mongodb';
import * as fs from "fs";

type Checker = {
    pos: number,
    "is_king": boolean,
    "stricken": boolean
    "color": "Black" | "White"
} | null | undefined

interface NodesItem {
    _id: ObjectId
    node: {
        cells: Checker[],
        "next_move": "White" | "Black"
        N: number,
        W: number
    },
    child: {
        cells: Checker[],
        "next_move": "White" | "Black"
        N: number,
        NN: number,
        W: number
    },
    repetitions: number
}

async function connectToMongo(): Promise<Db> {
    const url = 'mongodb://172.29.200.251:27017';
    const client = new MongoClient(url);


    try {
        await client.connect();
        console.log('Connected to MongoDB');

        const db = client.db('checkers');
        return db;
    } catch (error) {
        console.error('Error connecting to MongoDB', error);
        throw error;
    }
}

function cellsToArray(cells: Checker[], nextMove: "Black" | "White") {
    let [start, end, step] = nextMove == "Black" ?
        [cells.length - 1, -1, -1] : [0, cells.length, 1]
    let res = []
    for (let i = start; i !== end; i += step) {
        let checker = cells[i]
        let s = checker?.color ? (checker.color == "White" ? 1 : -1) : 0
        res.push(!checker ? 0 : s * (checker.is_king ? 1 : 0.3))
    }
    return res
}

async function mapCollection(db: Db): Promise<number[][]> {
    const collection: Collection = db.collection('nodes');

    try {
        const cursor = await collection.find().toArray();

        const transformedDocuments = cursor
            .map((document) => {
                const doc: NodesItem = {
                    _id: document._id,
                    node: document.item.node,
                    child: document.item.child,
                    repetitions: document.repetitions,
                };

                let x: { next_move: "Black" | "White", v_node: number, v_child: number, u: number } = {
                    u: 1.4 * Math.sqrt(Math.log(doc.node.N + doc.child.NN) / (1 + doc.child.N)),
                    v_node: doc.node.W / (1 + doc.node.N),
                    v_child: doc.child.W / (1 + doc.child.N),
                    next_move: doc.node.next_move
                }

                let v1 = cellsToArray(document.item.node.cells, document.item.node.next_move)
                let v2 = cellsToArray(document.item.child.cells, document.item.child.next_move)
                let v_norm = (x.v_child + 1) / 2
                return [...v1, ...v2, doc.node.next_move == "White" ? 1 : -1, x.u, v_norm]
            });
        return transformedDocuments

    } catch (error) {
        console.error('Error mapping collection', error);
        throw error;
    }
}


async function main() {
    let db = await connectToMongo();
    let list = await mapCollection(db)
    // list.sort((a, b) => a[65] - b[65])
    console.log("writing to file")
    fs.writeFileSync("vectors.json", JSON.stringify(list))
}

main().then(()=>console.log("finish"))