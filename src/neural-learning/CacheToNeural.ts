import * as fs from 'fs';
// @ts-ignore
import * as JSONStream from "jsonstream";
type Color =
    "Black"|
    "White"

type Vec = {cells: Array<{pos: number, color: Color, is_king: boolean} | null>, next_move: Color, W: number, N: number, NN: number }

let v:Vec[] | undefined = []
// Открытие потока чтения файла
const readStream = fs.createReadStream('cache.json', 'utf8');

// Создание парсера JSON
const jsonParser = JSONStream.parse('v.*');

// Обработка события "data" при поступлении данных
jsonParser.on('data', (data: { item: Vec; }) => {
    // Обработка каждого объекта данных по мере их получения
    // console.log(data);
    v!.push(data.item);
});

// Подключение парсера JSON к потоку чтения файла
readStream.pipe(jsonParser);

// Обработка события "error" при возникновении ошибки чтения файла
readStream.on('error', (err) => {
    console.error('Error reading file:', err);
});

// Обработка события "end" при завершении чтения файла
let rv: (number [])[] | undefined = []
readStream.on('end', () => {
    console.log('File reading completed.');
    v!.forEach((x)=>{
        let vv = x.cells.map((y)=> {
            if (!y) {
                return 0
            } else {
                let sign = y.color == "White" ? 1: -1;
                let pw = y.is_king ? 1 : 0.3
                return sign*pw
            }})
        vv.push((x.W/(x.N+1) +1)/2)
        vv.push(x.next_move=="White" ? 1: -1)
        rv!.push(vv)
    })


    v = undefined

    let r1 = JSON.stringify(rv)
    rv = undefined
    fs.writeFileSync('vector.json', r1);
});


