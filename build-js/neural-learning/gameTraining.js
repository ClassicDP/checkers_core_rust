"use strict";
Object.defineProperty(exports, "__esModule", { value: true });
const fs_1 = require("fs");
const checkers_core_1 = require("../../build-wasm/checkers_core");
const gameProcess_1 = require("../gameProcess");
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Перечисление возможных функций активации
var ActivationType;
(function (ActivationType) {
    ActivationType[ActivationType["Sigmoid"] = 0] = "Sigmoid";
    ActivationType[ActivationType["Relu"] = 1] = "Relu";
})(ActivationType || (ActivationType = {}));
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Класс нейронной сети
class NeuralNetwork {
    //-------------------------------------------------------------------------
    //Функция активации Сигмоида
    sigmoidActivation(value) {
        return 1.0 / (1 + Math.exp(-value));
    }
    //-------------------------------------------------------------------------
    //Функция активации ReLU
    reluActivation(value) {
        return Math.max(0, value);
    }
    //-------------------------------------------------------------------------
    //Определить тип активации нейронов
    defineActivation(activationType) {
        //Задать тип активации нейронов
        this.activationType = activationType;
        //Задать функцию активации нейронов
        switch (this.activationType) {
            case ActivationType.Sigmoid:
                this.activation = this.sigmoidActivation;
                break;
            case ActivationType.Relu:
                this.activation = this.reluActivation;
                break;
        }
    }
    //-------------------------------------------------------------------------
    //Создать нейронную сеть
    constructor(layerSizes = [], activationType = ActivationType.Sigmoid, initialWeightsRange = []) {
        //Список размеров слоев
        this.layerSizes = [];
        //Тип активации нейронов
        this.activationType = ActivationType.Sigmoid;
        //Список весов слоев
        this.layerWeights = [];
        //Пройти по списку слоев нейронной сети
        for (let layerIndex = 0; layerIndex < layerSizes.length; layerIndex++) {
            //Добавить размер текущего слоя в список размеров слоев
            this.layerSizes.push(layerSizes[layerIndex]);
        }
        //Определить тип активации нейронов
        this.defineActivation(activationType);
        //Пройти по списку слоев нейронной сети, исключая первый (потому что у него нет весов)
        for (let layerIndex = 1; layerIndex < this.layerSizes.length; layerIndex++) {
            //Создать веса текущего слоя
            const currentLayerWeights = new Array((this.layerSizes[layerIndex - 1] + 1) * this.layerSizes[layerIndex]);
            //Если задан диапазон начальных весов
            if (initialWeightsRange.length === 2) {
                //Пройти по весам текущего слоя
                for (let weightIndex = 0; weightIndex < currentLayerWeights.length; weightIndex++) {
                    //Задать случайный текущий вес текущего слоя
                    currentLayerWeights[weightIndex] = initialWeightsRange[0] + (initialWeightsRange[1] - initialWeightsRange[0]) * Math.random();
                }
            }
            else {
                //Пройти по весам текущего слоя
                for (let weightIndex = 0; weightIndex < currentLayerWeights.length; weightIndex++) {
                    //Задать нулевой вес текущего слоя
                    currentLayerWeights[weightIndex] = 0.0;
                }
            }
            //Добавить веса текущего слоя в список весов слоев
            this.layerWeights.push(currentLayerWeights);
        }
    }
    //-------------------------------------------------------------------------
    //загрузить нейронную сеть из файла
    loadFromFile(fileName) {
        //Открыть файл нейронной сети для чтения
        const fileHandle = (0, fs_1.openSync)(fileName, 'r');
        //Создать пустой буфер 
        let buffer;
        //Создать буфер количества слоев
        buffer = Buffer.alloc(2);
        //Прочитать из файла количество слоев
        (0, fs_1.readSync)(fileHandle, buffer);
        //Создать список размеров слоев
        this.layerSizes = new Array(buffer.readUint16BE(0));
        //Создать буфер размеров слоев
        buffer = Buffer.alloc(2 * this.layerSizes.length);
        //Прочитать из файла размеры слоев
        (0, fs_1.readSync)(fileHandle, buffer);
        //Пройти по списку размеров слоев
        for (let layerIndex = 0; layerIndex < this.layerSizes.length; layerIndex++) {
            //Задать размер текущего слоя
            this.layerSizes[layerIndex] = buffer.readUint16BE(2 * layerIndex);
        }
        //Создать буфер типа активации нейронов
        buffer = Buffer.alloc(1);
        //Прочитать из файла тип активации нейронов
        (0, fs_1.readSync)(fileHandle, buffer);
        //Определить тип активации нейронов
        this.defineActivation(buffer.readUint8(0));
        //Задать пустой список весов слоев
        this.layerWeights = [];
        //Пройти по списку весов слоев нейронной
        for (let layerWeightsIndex = 0; layerWeightsIndex < this.layerSizes.length - 1; layerWeightsIndex++) {
            //Создать буфер размера весов текущего слоя
            buffer = Buffer.alloc(2);
            //Прочитать из файла размера весов текущего слоя
            (0, fs_1.readSync)(fileHandle, buffer);
            //Создать веса текущего слоя
            const layerWeights = new Array(buffer.readUint16BE(0));
            //Создать буфер весов текущего слоя
            buffer = Buffer.alloc(4 * layerWeights.length);
            //Прочитать из файла размера весов текущего слоя
            (0, fs_1.readSync)(fileHandle, buffer);
            //Пройти по списку весов текущего слоя
            for (let weightIndex = 0; weightIndex < layerWeights.length; weightIndex++) {
                //Задать текущий вес
                layerWeights[weightIndex] = buffer.readFloatBE(4 * weightIndex);
            }
            //Добавить веса текущего слоя в список весов слоев
            this.layerWeights.push(layerWeights);
        }
        //Закрыть файл нейронной сети
        (0, fs_1.closeSync)(fileHandle);
    }
    //-------------------------------------------------------------------------
    //Сохранить нейронную сеть в файл
    saveToFile(fileName) {
        //Открыть файл нейронной сети для записи
        const fileHandle = (0, fs_1.openSync)(fileName, 'w');
        //Создать пустой буфер 
        let buffer;
        //Создать буфер количества слоев
        buffer = Buffer.alloc(2);
        //Записать в буфер количество слоев
        buffer.writeUInt16BE(this.layerSizes.length);
        //Записать в файл буфер количества слоев
        (0, fs_1.writeSync)(fileHandle, buffer);
        //Создать буфер размеров слоев
        buffer = Buffer.alloc(2 * this.layerSizes.length);
        //Пройти по списку размеров слоев
        for (let layerIndex = 0; layerIndex < this.layerSizes.length; layerIndex++) {
            //Записать в буфер размер текущего слоя
            buffer.writeUInt16BE(this.layerSizes[layerIndex], 2 * layerIndex);
        }
        //Записать в файл буфер размеров слоев
        (0, fs_1.writeSync)(fileHandle, buffer);
        //Создать буфер типа активации нейронов
        buffer = Buffer.alloc(1);
        //Записать в буфер тип активации нейронов
        buffer.writeUInt8(this.activationType);
        //Записать в файл буфер типа активации нейронов
        (0, fs_1.writeSync)(fileHandle, buffer);
        //Пройти по списку весов слоев
        for (let layerWeightsIndex = 0; layerWeightsIndex < this.layerWeights.length; layerWeightsIndex++) {
            //Создать буфер размера весов текущего слоя
            buffer = Buffer.alloc(2);
            //Записать в буфер размер весов текущего слоя
            buffer.writeUint16BE(this.layerWeights[layerWeightsIndex].length);
            //Записать в файл буфер размера весов текущего слоя
            (0, fs_1.writeSync)(fileHandle, buffer);
            //Создать буфер весов текущего слоя
            buffer = Buffer.alloc(4 * this.layerWeights[layerWeightsIndex].length);
            //Пройти по списку весов текущего слоя
            for (let weightIndex = 0; weightIndex < this.layerWeights[layerWeightsIndex].length; weightIndex++) {
                //Записать в буфер весов текущего слоя текущий вес
                buffer.writeFloatBE(this.layerWeights[layerWeightsIndex][weightIndex], weightIndex * 4);
            }
            //Записать в файл буфер весов текущего слоя
            (0, fs_1.writeSync)(fileHandle, buffer);
        }
        //Закрыть файл нейронной сети
        (0, fs_1.closeSync)(fileHandle);
    }
    //-------------------------------------------------------------------------
    //Сделать вывод нейронной сети
    predict(input) {
        //Задать выход предыдущего слоя нейронной сети
        let previousLayerOutput = [];
        //Задать выход текущего слоя нейронной сети
        let currentLayerOutput = [];
        //Создать выход предыдущего слоя нейронной сети равным входным данным
        previousLayerOutput = new Array(input.length);
        //Пройти по входным данным нейронной сети
        for (let i = 0; i < input.length; i++) {
            //Скопировать в выход предыдущего слоя нейронной сети текущее значение входных данных
            previousLayerOutput[i] = input[i];
        }
        //Пройти по списку весов нейронной сети
        for (let layerWeightsIndex = 0; layerWeightsIndex < this.layerWeights.length; layerWeightsIndex++) {
            //Создать выход текущего слоя нейронной сети равным размеру текущего слоя
            currentLayerOutput = new Array(this.layerSizes[layerWeightsIndex + 1]);
            //Пройти по выходу текущего слоя нейронной сети
            for (let i = 0; i < currentLayerOutput.length; i++) {
                //Вычислить шаг весов отдельных нейроннов в текущем слое
                const weightsStride = this.layerWeights[layerWeightsIndex].length / currentLayerOutput.length;
                //Задать выход текущего нейрона выхода текущего слоя нейронной сети равным смещению текущего нейрона
                currentLayerOutput[i] = this.layerWeights[layerWeightsIndex][weightsStride * i];
                //Пройти по выходу предыдущего слоя нейронной сети
                for (let j = 0; j < previousLayerOutput.length; j++) {
                    //Добавить к выходу текущего нейрона выхода текущего слоя нейронной сети взвешенный выход текущего нейрона выхода предыдущего слоя
                    currentLayerOutput[i] += this.layerWeights[layerWeightsIndex][weightsStride * i + j + 1] * previousLayerOutput[j];
                }
                //Задать выход текущего нейрона выхода текущего слоя нейронной сети равным значению функции активации
                currentLayerOutput[i] = this.activation(currentLayerOutput[i]);
            }
            //Если текущий слой не последний в нейронной сети
            if (layerWeightsIndex !== this.layerWeights.length - 1) {
                //Обменять местами вывод предыдущего и текущего слоя нейронной сети
                [previousLayerOutput, currentLayerOutput] = [currentLayerOutput, previousLayerOutput];
            }
        }
        //Вернуть вывод текущего слоя нейронной сети
        return currentLayerOutput;
    }
    //-------------------------------------------------------------------------
    //Получить размеры слоев нейронной сети
    getLayerSizes() {
        //Создать список размеров слоев нейронной сети
        const layerSizes = new Array(this.layerSizes.length);
        //Пройти по списку размеров слоев нейронной сети
        for (let layerIndex = 0; layerIndex < this.layerSizes.length; layerIndex++) {
            //Скопировать размер текущего слоя нейронной сети
            layerSizes[layerIndex] = this.layerSizes[layerIndex];
        }
        //Вернуть список размеров слоев нейронной сети
        return layerSizes;
    }
    //-------------------------------------------------------------------------
    //Получить количество слоев нейронной сети
    getLayersCount() {
        return this.layerSizes.length;
    }
    //-------------------------------------------------------------------------
    //Получить тип активации нейронов
    getActivationType() {
        return this.activationType;
    }
    //-------------------------------------------------------------------------
    //Получить веса заданного слоя
    getLayerWeights(layerIndex) {
        //Создать список весов заданного слоя
        const layerWeights = new Array(this.layerWeights[layerIndex - 1].length);
        //Пройти по весам заданного слоя
        for (let weightIndex = 0; weightIndex < layerWeights.length; weightIndex++) {
            //Скопировать текущий вес заданного слоя
            layerWeights[weightIndex] = this.layerWeights[layerIndex - 1][weightIndex];
        }
        //Вернуть список весов заданного слоя
        return layerWeights;
    }
    //-------------------------------------------------------------------------
    //Задать веса заданного слоя
    setLayerWeights(layerIndex, weights) {
        //Пройти по весам заданного слоя
        for (let weightIndex = 0; weightIndex < this.layerWeights[layerIndex - 1].length; weightIndex++) {
            //Скопировать текущий вес заданного слоя
            this.layerWeights[layerIndex - 1][weightIndex] = weights[weightIndex];
        }
    }
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Класс элемента набора данных
class DatasetItem {
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Класс элемента популяции
class PopulationItem {
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Класс параметров тренировки
class TrainingParams {
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Класс параметров эволюции
class EvolutionParams {
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Сформировать набор данных
function formDataset(datasetItems, gamesCount, verbose) {
    //Задать нулевое количество новых элементов набора данных
    let newDatasetItemsCount = 0;
    //Пройти по играм
    for (let gameNumber = 0; gameNumber < gamesCount; gameNumber++) {
        //Если установлен флаг вывода на консоль процесса формирования набора данных
        if (verbose) {
            //Вывести на консоль сообщение о начале игры для текущего элемента популяции
            console.log(`Playing game #${gameNumber + 1} ...`);
        }
        let listOrFinish;
        const gameProcess = new gameProcess_1.GameProcess(8);
        gameProcess.game.set_mcts_lim(200000);
        const whitePosList = [0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22];
        const blackPosList = whitePosList.map(x => 63 - x);
        whitePosList.forEach(x => gameProcess.insertPiece(x, checkers_core_1.Color.White, false));
        blackPosList.forEach(x => gameProcess.insertPiece(x, checkers_core_1.Color.Black, false));
        gameProcess.moveColor = checkers_core_1.Color.White;
        let movesCount = 0;
        do {
            //Получить список возможных позиций для ходов
            listOrFinish = gameProcess.game.find_mcts_and_make_best_move_ts_n(true);
            //Увеличить количество сделанных ходов
            movesCount++;
            //Если есть список возможных ходов (игра еще не завершилась)
            if (listOrFinish instanceof Array) {
                //Задать лучшую позицию равной первой
                let bestPositionIndex = 0;
                //Задать нулевую оценку лучшей позиции
                let bestPositionScore = 0;
                //Пройти по списку возможных позиций для ходов
                for (let positionIndex = 0; positionIndex < listOrFinish.length; positionIndex++) {
                    //Создать нормализованную позицию
                    const normalizedPosition = new Array(listOrFinish[positionIndex].length - 2);
                    //Пройти по элементам текущей позиции
                    for (let i = 0; i < normalizedPosition.length; i++) {
                        //Нормализовать текущее значение текущей позиции
                        normalizedPosition[i] = (listOrFinish[positionIndex][i] + 3) / 6;
                    }
                    //Вычислить качество позиции
                    const positionScore = listOrFinish[positionIndex][32] / listOrFinish[positionIndex][33];
                    //Сбросить флаг существования элемента набора данных
                    let datasetItemExists = false;
                    //Пройти по элементам набора данных
                    for (let i = 0; i < datasetItems.length; i++) {
                        //Установить флаг равности позиций
                        let positionsEqual = true;
                        //Пройти по элементам позиции текущего элемента набора данных
                        for (let j = 0; j < datasetItems[i].position.length; j++) {
                            //Если текущие элементы позиции текущего элемента набора данных не совпадают
                            if (datasetItems[i].position[j] !== normalizedPosition[j]) {
                                //Сбросить флаг равности позиций
                                positionsEqual = false;
                                //Закончить сравнение позиций
                                break;
                            }
                        }
                        //Если позиции равны
                        if (positionsEqual) {
                            //Установить флаг существования элемента набора данных
                            datasetItemExists = true;
                            //Закончить поиск
                            break;
                        }
                    }
                    //Если элемент набора данных не существует
                    if (!datasetItemExists) {
                        //Добавить новый элемент набора данных
                        datasetItems.push({ position: normalizedPosition, score: positionScore });
                        //Увеличить количество новых элементов набора данных
                        newDatasetItemsCount++;
                    }
                    //Если оценка текущей позиции лучше оценки лучшей позиции
                    if (positionScore > bestPositionScore) {
                        //Задать оценку лучшей позиции равной оценке текущей позиции
                        bestPositionScore = positionScore;
                        //Задать лучшую позицию равной текущей
                        bestPositionIndex = positionIndex;
                    }
                }
                //Сделать ход, соответствующий лучшей позиции
                gameProcess.game.move_by_tree_index_ts_n(bestPositionIndex);
                //Увеличить количество сделанных ходов
                movesCount++;
            }
        } while (listOrFinish instanceof Array);
        //Если установлен флаг вывода на консоль процесса формирования набора данных
        if (verbose) {
            //Вывести на консоль сообщение о результате игры
            console.log(`Game ended. ${listOrFinish} in ${movesCount} moves`);
        }
    }
    //Вернуть количество новых элементов набора данных
    return newDatasetItemsCount;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Загрузить набор данных
function loadDataset(fileName) {
    //Задать пустой список элементов набора данных
    let datasetItems = [];
    //Если файл набора данных существует
    if ((0, fs_1.existsSync)(fileName)) {
        //Открыть файл набора данных для чтения
        const datasetFileHandle = (0, fs_1.openSync)(fileName, 'r');
        //Строка элементов набора данных
        let datasetItemsString = '';
        //Читать файл набора данных
        while (1) {
            //Создать буфер для чтения
            let buffer = Buffer.alloc(1024);
            //Прочитать часть файла набора данных
            const bytesRead = (0, fs_1.readSync)(datasetFileHandle, buffer);
            //Уменьшить размер буфера до количества прочитанных байтов
            buffer = buffer.subarray(0, bytesRead);
            //Если хотя бы какие-то данные были прочитаны из файла (конец файла не достугнут)
            if (buffer.length) {
                //Добавить прочитанные данные к строке элементов набора данных
                datasetItemsString += buffer.toString();
            }
            else {
                //Завершить чтение
                break;
            }
        }
        //Закрыть файл набора данных
        (0, fs_1.closeSync)(datasetFileHandle);
        //Преобразовать строку элементов набора данных в список элементов набора данных
        datasetItems = JSON.parse(datasetItemsString);
    }
    //Вернуть элементы набора данных
    return datasetItems;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Сохранить набор данных
function saveDataset(datasetItems, fileName) {
    //Открыть файл набора данных для записи
    const datasetFileHandle = (0, fs_1.openSync)(fileName, 'w');
    //Записать в файл набора данных элементы набора данных
    (0, fs_1.writeSync)(datasetFileHandle, JSON.stringify(datasetItems));
    //Закрыть файл набора данных
    (0, fs_1.closeSync)(datasetFileHandle);
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Создать популяцию
function createPopulation(size, networkLayerSizes, activationType, initialWeightsRange) {
    //Создать популяцию нейронных сетей
    const population = new Array(size);
    //Пройти по элементам популяции нейроных сетей
    for (let populationItemIndex = 0; populationItemIndex < population.length; populationItemIndex++) {
        //Поместить нейронную сеть в популяцию нейронных сетей
        population[populationItemIndex] =
            {
                network: new NeuralNetwork(networkLayerSizes, activationType, initialWeightsRange),
                quality: 0
            };
    }
    //Вернуть популяцию нейронных сетей
    return population;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Загрузить популяцию
function loadPopulation(path, trainingInfoFileName) {
    //Создать пустую популяцию нейронных сетей
    let population = [];
    //Задать нулевое значение количества поколений популяции нейронных сетей
    let passedGenerationsCount = 0;
    //Если каталог популяции существует
    if ((0, fs_1.existsSync)(path)) {
        //Получить список файлов популяции
        const fileNames = (0, fs_1.readdirSync)(path);
        //Пройти по именам файлов популяции
        for (let i = 0; i < fileNames.length; i++) {
            //Если текущий файл не является файлом информации о тренировке
            if (fileNames[i] === trainingInfoFileName) {
                //Открыть файл информации о тренировке для чтения
                const trainingInfoFileHandle = (0, fs_1.openSync)(path + '/' + trainingInfoFileName, 'r');
                //Создать буфер для чтения
                let buffer = Buffer.alloc(30);
                //Прочитать файл информации о тренировке
                const bytesRead = (0, fs_1.readSync)(trainingInfoFileHandle, buffer);
                //Уменьшить размер буфера до количества прочитанных байтов
                buffer = buffer.subarray(0, bytesRead);
                //Задать значение количества поколений популяции нейронных сетей
                passedGenerationsCount = JSON.parse(buffer.toString()).generation;
                //Закрыть файл информации о тренировке
                (0, fs_1.closeSync)(trainingInfoFileHandle);
            }
            else {
                //Создать элемент популяции нейронных сетей
                const populationItem = {
                    network: new NeuralNetwork(),
                    quality: 0
                };
                //загрузить нейронную сеть из файла
                populationItem.network.loadFromFile(path + '/' + fileNames[i]);
                //Добавить элемент популяции в популяцию нейронных сетей
                population.push(populationItem);
            }
        }
    }
    //Вернуть результат загрузки популяции нейронных сетей
    return { population, passedGenerationsCount };
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Сохранить популяцию
function savePopulation(generation, population, path, trainingInfoFileName) {
    //Если путь каталог сохранения популяции не существует
    if (!(0, fs_1.existsSync)(trainingParams.savingPath)) {
        //Создать каталог сохранения популяции
        (0, fs_1.mkdirSync)(trainingParams.savingPath);
    }
    //Открыть файл информации о тренировке для записи
    const trainingInfoFileHandle = (0, fs_1.openSync)(path + '/' + trainingInfoFileName, 'w');
    //Записать в файл информации о тренировке номер поколения популяции
    (0, fs_1.writeSync)(trainingInfoFileHandle, JSON.stringify({ generation }));
    //Закрыть файл информации о тренировке
    (0, fs_1.closeSync)(trainingInfoFileHandle);
    //Пройти по элементам популяции
    for (let populationItemIndex = 0; populationItemIndex < population.length; populationItemIndex++) {
        //Сохранить текущий элемент популяции в файл
        population[populationItemIndex].network.saveToFile(trainingParams.savingPath + '/' + populationItemIndex);
    }
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Оценить качество популяции
function estimatePopulationQuality(population, gamesPerPopulationItem, verbose) {
    //Задать нулевое качество популяции
    let populationQuality = 0;
    //Пройти по элементам популяции
    for (let populationItemIndex = 0; populationItemIndex < population.length; populationItemIndex++) {
        //Если установлен флаг вывода на консоль процесса обучения
        if (verbose) {
            //Вывести на консоль сообщение о начале оценки качества текущего элемента популяции
            console.log(`Evaluating quality of population item #${populationItemIndex + 1} ...`);
        }
        //Задать нулевое количество очков, набранных текущим элементом популяции в играх
        let gamesScore = 0;
        //Задать нулевую сумму квадратов ошибок
        let sumOfSquaredErrors = 0;
        //Пройти по играм для текущего элемента популяции
        for (let gameNumber = 0; gameNumber < gamesPerPopulationItem; gameNumber++) {
            //Если установлен флаг вывода на консоль процесса обучения
            if (verbose) {
                //Вывести на консоль сообщение о начале игры для текущего элемента популяции
                console.log(`Playing game #${gameNumber + 1} ...`);
            }
            let listOrFinish;
            let gameProcess = new gameProcess_1.GameProcess(8);
            gameProcess.game.set_depth(1); // <-- depth limit
            let whitePosList = [0, 2, 4, 6, 9, 11, 13, 15, 16, 18, 20, 22];
            let blackPosList = whitePosList.map(x => 63 - x);
            whitePosList.forEach(x => gameProcess.insertPiece(x, checkers_core_1.Color.White, false));
            blackPosList.forEach(x => gameProcess.insertPiece(x, checkers_core_1.Color.Black, false));
            gameProcess.moveColor = checkers_core_1.Color.White;
            let movesCount = 0;
            do {
                //Получить список возможных позиций для ходов
                listOrFinish = gameProcess.game.find_mcts_and_make_best_move_ts_n(true);
                //Увеличить количество сделанных ходов
                movesCount++;
                //Если есть список возможных ходов (игра еще не завершилась)
                if (listOrFinish instanceof Array) {
                    //Задать лучшую позицию равной первой
                    let bestPositionIndex = 0;
                    //Задать нулевую оценку лучшей позиции
                    let bestPositionScore = 0;
                    //Пройти по списку возможных позиций для ходов
                    for (let positionIndex = 0; positionIndex < listOrFinish.length; positionIndex++) {
                        //Создать нормализованную позицию
                        const normalizedPosition = new Array(listOrFinish[positionIndex].length - 2);
                        //Пройти по элементам текущей позиции
                        for (let i = 0; i < normalizedPosition.length; i++) {
                            //Нормализовать текущее значение текущей позиции
                            normalizedPosition[i] = (listOrFinish[positionIndex][i] + 3) / 6;
                        }
                        //Оценить текущую позицию
                        const positionScore = population[populationItemIndex].network.predict(normalizedPosition)[0];
                        //Увеличить сумму квадратов ошибок на квадрат разницы между предсказанной и действительной оценкой позиции
                        sumOfSquaredErrors += Math.pow((listOrFinish[positionIndex][32] / listOrFinish[positionIndex][33] - positionScore), 2);
                        //Если оценка текущей позиции лучше оценки лучшей позиции
                        if (positionScore > bestPositionScore) {
                            //Задать оценку лучшей позиции равной оценке текущей позиции
                            bestPositionScore = positionScore;
                            //Задать лучшую позицию равной текущей
                            bestPositionIndex = positionIndex;
                        }
                    }
                    //Сделать ход, соответствующий лучшей позиции
                    gameProcess.game.move_by_index_ts_n(bestPositionIndex);
                    //Увеличить количество сделанных ходов
                    movesCount++;
                }
            } while (listOrFinish instanceof Array);
            //Если игра завершилась поражением
            if (listOrFinish.startsWith('White')) {
                //Увеличить количество очков, набранных текущим элементом популяции, соответствующем поражению
                gamesScore += 0;
            }
            //Если игра завершилась ничьей
            else if (listOrFinish.startsWith('Draw')) {
                //Увеличить количество очков, набранных текущим элементом популяции, соответствующем ничьей
                gamesScore += 1;
            }
            //Если игра завершилась победой
            else if (listOrFinish.startsWith('Black')) {
                //Увеличить количество очков, набранных текущим элементом популяции, соответствующем победе
                gamesScore += 2;
            }
            //Если установлен флаг вывода на консоль процесса обучения
            if (verbose) {
                //Вывести на консоль сообщение о результате игры для текущего элемента популяции
                console.log(`Game ended. ${listOrFinish} in ${movesCount} moves`);
            }
        }
        //Вычислить качество элемента популяции
        population[populationItemIndex].quality = 100 / (sumOfSquaredErrors + 0.01);
        //Если установлен флаг вывода на консоль процесса обучения
        if (verbose) {
            //Вывести на консоль сообщение о набранных в играх очках текущего элемента популяции
            console.log(`Population item games score = ${gamesScore}`);
            //Вывести на консоль сообщение о качестве текущего элемента популяции
            console.log(`Population item quality = ${population[populationItemIndex].quality}`);
        }
    }
    //Пройти по элементам популяции
    for (let populationItemIndex = 0; populationItemIndex < population.length; populationItemIndex++) {
        //Увеличить качество популяции на качество текущего элемента популяции
        populationQuality += population[populationItemIndex].quality;
    }
    //Вычислить качество популяции как среднее значение качеств элементов популяции
    populationQuality /= population.length;
    //Вернуть качество популяции
    return populationQuality;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Эволюционировать популяцию
function evolvePopulation(population, evolutionParams) {
    //Создать эволюционировавшую популяция
    const evolvedPopulation = new Array(population.length);
    //Пройти по элементам популяции
    for (let populationItemIndex = 0; populationItemIndex < population.length; populationItemIndex++) {
        //Добавить в эволюционировавшую популяцию новый элемент популяции
        evolvedPopulation[populationItemIndex] =
            {
                network: new NeuralNetwork(population[populationItemIndex].network.getLayerSizes(), population[populationItemIndex].network.getActivationType()),
                quality: 0
            };
    }
    //Упорядочить популяцию в порядке убывания качества элементов
    population.sort((item1, item2) => { return item2.quality - item1.quality; });
    //Вычислить количество элементов, которые будут отбираться в эволюционировавшую популяцию
    const selectedPopulationItemsCount = Math.round(evolutionParams.selectionFactor * population.length);
    //Пройти по элементам эволюционировавшей популяции
    for (let populationItemIndex = 0; populationItemIndex < evolvedPopulation.length; populationItemIndex++) {
        //Сгенерировать индекс случайно выбранного элемента популяции для отбора в эволюционировавшую популяцию
        const selectedPopulationItemIndex = Math.round(Math.random() * (selectedPopulationItemsCount - 1));
        //Пройти по слоям текущего элемента эволюционировавшей популяции
        for (let layerIndex = 1; layerIndex < evolvedPopulation[populationItemIndex].network.getLayersCount(); layerIndex++) {
            //Задать веса текущего слоя текущего элемента эволюционировавшей популяции равными весам случайно выбранного элемента прежней популяции
            evolvedPopulation[populationItemIndex].network.setLayerWeights(layerIndex, population[selectedPopulationItemIndex].network.getLayerWeights(layerIndex));
        }
    }
    //Пройти по парам элементов популяции (перекрестное скрещивание)
    for (let pairIndex = 0; pairIndex < evolvedPopulation.length / 2; pairIndex++) {
        //Пройти по слоям первого элемента пары эволюционировавшей популяции
        for (let layerIndex = 1; layerIndex < evolvedPopulation[2 * pairIndex].network.getLayersCount(); layerIndex++) {
            //Получить веса текущего слоя первого элемента пары
            const firstPopulationItemLayerWeights = evolvedPopulation[2 * pairIndex].network.getLayerWeights(layerIndex);
            //Получить веса текущего слоя второго элемента пары
            const secondPopulationItemLayerWeights = evolvedPopulation[2 * pairIndex + 1].network.getLayerWeights(layerIndex);
            //Сгенерировать индекс разбиения весов
            const weightsBreakingIndex = Math.round(Math.random() * (firstPopulationItemLayerWeights.length - 1));
            //Пройти по весам первого блока разбиения текущего слоя
            for (let weightIndex = 0; weightIndex < weightsBreakingIndex; weightIndex++) {
                //Обменять местами текущие веса первого и второго элемента пары эволюционировавшей популяции
                [firstPopulationItemLayerWeights[weightIndex], secondPopulationItemLayerWeights[weightIndex]] =
                    [secondPopulationItemLayerWeights[weightIndex], firstPopulationItemLayerWeights[weightIndex]];
            }
            //Пройти по весам второго блока разбиения текущего слоя
            for (let weightIndex = weightsBreakingIndex; weightIndex < firstPopulationItemLayerWeights.length; weightIndex++) {
                //Обменять местами текущие веса первого и второго элемента пары эволюционировавшей популяции
                [firstPopulationItemLayerWeights[weightIndex], secondPopulationItemLayerWeights[weightIndex]] =
                    [secondPopulationItemLayerWeights[weightIndex], firstPopulationItemLayerWeights[weightIndex]];
            }
            //Задать веса первого элемента пары эволюционировавшей популяции
            evolvedPopulation[2 * pairIndex].network.setLayerWeights(layerIndex, firstPopulationItemLayerWeights);
            //Задать веса второго элемента пары эволюционировавшей популяции
            evolvedPopulation[2 * pairIndex + 1].network.setLayerWeights(layerIndex, secondPopulationItemLayerWeights);
        }
    }
    //Пройти по элементам эволюционировавшей популяции (мутация)
    for (let populationItemIndex = 0; populationItemIndex < evolvedPopulation.length; populationItemIndex++) {
        //Пройти по весам текущего элемента эволюционировавшей популяции
        for (let layerIndex = 1; layerIndex < evolvedPopulation[populationItemIndex].network.getLayersCount(); layerIndex++) {
            //Получить веса текущего слоя текущего элемента эволюционировавшей популяции
            const layerWeights = evolvedPopulation[populationItemIndex].network.getLayerWeights(layerIndex);
            //Пройти по весам текущего слоя текущего элемента эволюционировавшей популяции
            for (let weightIndex = 0; weightIndex < layerWeights.length; weightIndex++) {
                //Если следует применять мутацию
                if (Math.random() <= evolutionParams.mutationFactor) {
                    //Изменить текущий вес текущего слоя текущего элемента эволюционировавшей популяции
                    layerWeights[weightIndex] += -evolutionParams.mutationStrength + 2 * evolutionParams.mutationStrength * Math.random();
                }
            }
            //Задать веса текущего слоя текущего элемента эволюционировавшей популяции
            evolvedPopulation[populationItemIndex].network.setLayerWeights(layerIndex, layerWeights);
        }
    }
    //Вернуть эволюционировавшую популяцию
    return evolvedPopulation;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Тренировать нейронную сеть генетическим алгоритмом
function geneticTrain(trainingParams) {
    //Имя файла информации о популяции нейронных сетей
    const trainingInfoFileName = 'training.json';
    //Создать популяцию нейронных сетей
    let population = [];
    //Задать нулевое значение количества поколений популяции нейронных сетей
    let passedGenerationsCount = 0;
    //Загрузить популяцию нейронных сетей
    const populationInfo = loadPopulation(trainingParams.savingPath, trainingInfoFileName);
    //Если размер популяции нейронных сетей корректный
    if (populationInfo.population.length === trainingParams.populationSize) {
        //Задать популяцию нейронных сетей соответствующей загруженной
        population = populationInfo.population;
        //Задать значение количества поколений популяции нейронных сетей соответствующим загруженному
        passedGenerationsCount = populationInfo.passedGenerationsCount;
        //Вывести на консоль сообщение об успешной загрузке популяции нейронных сетей
        console.log(`Population loaded. Training will be continued from generation #${passedGenerationsCount}`);
    }
    else {
        //Создать популяцию нейронных сетей заново
        population = createPopulation(trainingParams.populationSize, trainingParams.layerSizes, trainingParams.activationType, trainingParams.initialWeightsRange);
        //Задать нулевое значение количества поколений популяции нейронных сетей
        passedGenerationsCount = 0;
        //Вывести на консоль сообщение об успешной загрузке популяции нейронных сетей
        console.log('Could not load population! New population was created');
    }
    //Получить время начала тренировки
    const trainingStartTime = Date.now();
    //Задать количество поколений с последнего сохранения популяции нейронных сетей
    let passedGenerationsCountSinceLastSaving = 0;
    //Пройти по поколениям популяции нейронных сетей
    for (let generation = 0; generation < trainingParams.generationsCount; generation++) {
        //Если установлен флаг вывода на консоль процесса обучения
        if (trainingParams.verbose) {
            //Вывести на консоль сообщение о начале оценки качества популяции
            console.log(`Evaluating population quality of generation #${passedGenerationsCount + generation + 1} ...`);
        }
        //Оценить качество популяции нейронных сетей
        const populationQuality = estimatePopulationQuality(population, trainingParams.gamesPerPopulationItem, trainingParams.verbose);
        //Если установлен флаг вывода на консоль процесса обучения
        if (trainingParams.verbose) {
            //Вывести на консоль качество популяции нейронных сетей
            console.log(`Population quality = ${populationQuality}`);
        }
        //Задать параметры эволюции популяции нейронных сетей
        const evolutionParams = {
            selectionFactor: trainingParams.selectionFactor,
            mutationFactor: trainingParams.mutationFactor,
            mutationStrength: trainingParams.mutationStrength
        };
        //Эволюционировать популяцию нейронных сетей
        population = evolvePopulation(population, evolutionParams);
        //Увеличить количество поколений с последнего сохранения популяции
        passedGenerationsCountSinceLastSaving++;
        //Если прошло достаточное количество поколений с последнего сохранения популяции нейронных сетей
        if (passedGenerationsCountSinceLastSaving === trainingParams.savingInterval) {
            //Сохранить популяцию нейронных сетей
            savePopulation(passedGenerationsCount + generation + 1, population, trainingParams.savingPath, trainingInfoFileName);
            //Сбросить количество поколений с последнего сохранения популяции нейронных сетей
            passedGenerationsCountSinceLastSaving = 0;
        }
    }
    //Оценить финальное качество популяции нейронных сетей
    const finalPopulationQuality = estimatePopulationQuality(population, trainingParams.gamesPerPopulationItem, trainingParams.verbose);
    //Упорядочить популяцию нейронных сетей в порядке убывания качества элементов популяции
    population.sort((item1, item2) => { return item2.quality - item1.quality; });
    //Получить время конца тренировки
    const trainingEndTime = Date.now();
    //Вывести на консоль финальное качество популяции нейронных сетей и качество лучшей нейронной сети
    console.log(`Final population quality = ${finalPopulationQuality}, best neural network quality = ${population[0].quality}`);
    //Вывести на консоль время тренировки
    console.log(`Training time = ${(trainingEndTime - trainingStartTime) / 1000}s`);
    //Вернуть лучшую нейронную сеть в популяции нейронных сетей
    return population[0].network;
}
//-----------------------------------------------------------------------------
//-----------------------------------------------------------------------------
//Параметры тренировки
const trainingParams = {
    populationSize: 10,
    generationsCount: 800,
    gamesPerPopulationItem: 5,
    layerSizes: [32, 60, 30, 1],
    activationType: ActivationType.Sigmoid,
    initialWeightsRange: [-0.2, 0.2],
    selectionFactor: 0.7,
    mutationFactor: 0.1,
    mutationStrength: 1,
    savingPath: 'population',
    savingInterval: 1,
    verbose: true
};
//Тренировать нейронную сеть генетическим алгоритмом
//geneticTrain(trainingParams)
const datasetFileName = 'dataset.json';
let datasetItems = loadDataset(datasetFileName);
console.log(`Dataset size = ${datasetItems.length}`);
const newDatasetItemsCount = formDataset(datasetItems, 1000, true);
console.log(`New dataset items count = ${newDatasetItemsCount}`);
saveDataset(datasetItems, datasetFileName);
//# sourceMappingURL=gameTraining.js.map