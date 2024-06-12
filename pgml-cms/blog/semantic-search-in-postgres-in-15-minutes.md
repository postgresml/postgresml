
---
description: >-
  Learn how to implement semantic search in postgres with nothing but SQL.
featured: true
image: ".gitbook/assets/image (2) (2).png"
tags: ["Engineering"]
---

# Semantic Search in Postgres in 15 Minutes

<div align="left">

<figure><img src=".gitbook/assets/silas.jpg" alt="Author" width="125"><figcaption></figcaption></figure>

</div>

Silas Marvin

June 15, 2024

## What is and is not Semantic Search

Semantic search is a new form of machine learning powered search that doesn’t rely on any form of keyword matching, but transforms text into embeddings and performs nearest neighbors search. 

It is not a complete replacement for full text search. In many cases full text search is capable of outperforming semantic search. Specifically, if a user knows the exact phrase in a document they want to match, full text search is faster and guaranteed to return the correct result while semantic search is only likely to return the correct result. Full text search and semantic search can be combined to create powerful and robust search systems.

Semantic search is not just for machine learning engineers. The actual system behind semantic search is relatively easy to implement and thanks to new Postgres extensions like pgml and pgvector, is readily available to SQL developers. Just as it is expected for modern SQL developers to be familiar with and capable of implementing full text search, soon SQL developers will be expected to implement semantic search.

## Embeddings 101

Semantic search is powered by embeddings. To understand how semantic search works, we must have a basic understanding of embeddings. 

Embeddings are vectors. Given some text and some embedding model, we can convert text to vectors:

!!! generic

!!! code_block time="10.493 ms"

```postgresql
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'test');
```

!!! 

!!! results

```text
{0.100442916,0.19116051,0.031572577,0.4392428,-0.4684339,0.2671335,0.000183642,0.402029,0.52400684,0.9102567,0.3289586,0.13825876,0.060483586,-0.9315942,-0. 36723328,-0.34614784,-0.5203485,-0.0026301902,-0.4359727,-0.2078317,-0.3624561,0.151347,-1.0850769,0.03073607,-0.38463902,0.5746146,-0.065853976,-0.02959722, 0.3181093,0.60477436,-0.20975977,-0.112029605,0.32066098,-0.92783266,0.17003687,-0.6294021,0.94078255,-0.32587636,-0.06733026,-0.41903132,-0.29940385,-0.0473 7147,0.7173805,-0.4461215,-1.2713308,-0.44129986,-0.46632472,-0.89888424,-0.22231846,-0.34233224,0.09881798,0.17341912,0.27128124,-0.7020756,0.113429464,-0.2 2964618,-0.22798245,0.1105072,-0.8441625,1.238711,0.8674123,-0.14600402,0.391594,-0.9928256,0.24864249,-0.11477054,0.23513256,-0.366138,-0.13302355,-0.449127 64,-0.45309332,0.4775117,-0.19158679,-0.6634198,-0.21402365,0.2285473,0.09665201,0.47793895,-0.456355,0.33732873,0.2820914,0.17230554,0.14925064,0.23560016,- 1.2823433,-0.8188313,0.07958572,0.758208,0.39241728,-0.021326406,-0.0026611753,0.4960972,-0.5743201,-0.10779899,-0.53800243,0.11743014,0.17272875,-0.537756,0 .15774709,-0.024241826,0.75601554,0.5569049,-0.098995246,1.0593386,-0.90425104,0.3956237,0.024354521,-0.32476613,-0.5950871,-0.75371516,-0.31561607,0.0696320 8,0.6516349,0.5434117,-0.7673086,0.7324196,0.15175763,1.1354275,-0.56910944,-0.09738582,0.35705066,0.018214416,-0.091416046,-0.19074695,-0.34592274,-0.115972 71,-0.5033031,0.6735635,-0.05835747,-0.21572702,-0.58285874,0.095334634,0.8742985,0.6349386,0.4706169,-0.029405594,-0.50637966,0.4569466,0.2924249,-0.9321604 ,0.34013036,1.1258447,-0.28096777,1.2910426,0.32090122,0.5956652,0.22290495,0.08063537,-0.3783538,0.71436244,-0.90230185,-0.4399799,0.24639784,0.3069413,-0.4 8032463,0.27206388,-0.43469447,-0.2339563,0.12732148,0.22685277,-0.7924011,0.3359629,-0.30172998,0.43736732,-0.521733,1.324045,-0.28834093,-0.15974034,0.2684 1715,-0.33593872,0.73629487,-0.1049053,0.16749644,0.3264093,-0.101803474,0.22606595,1.2974273,0.22830595,0.39088526,0.4486965,-0.57037145,-0.09293561,-0.0394 99372,0.47220317,0.74698365,0.2392712,0.23049277,-0.52685314,-0.5007666,-0.03302433,-0.2098883,0.47145832,-0.6392486,0.58358306,-0.15019979,0.32308426,-0.506 2344,-0.16731891,-0.55598915,-1.7701503,-0.3798853,0.54786783,-0.71652645,-0.1773671,0.2289979,-1.0015582,0.5309544,0.81240565,-0.17937969,-0.3966378,0.60281 52,0.8962739,-0.176342,-0.010436469,0.02249392,0.09129296,-0.105494745,0.970157,-0.26875457,0.10241943,0.6148784,-0.35458624,0.5211534,0.61402124,0.48477444, -0.16437246,-0.28179103,1.2025942,-0.22813843,-0.09890138,0.043852188,1.0050704,-0.17958593,1.3325024,0.59157765,0.4212076,1.0721352,0.095619194,0.26288888,0 .42549643,0.2535346,0.35668525,0.82613224,0.30157906,-0.567903,0.32422608,-0.046756506,0.08393835,-0.31040233,0.7402205,0.7880251,0.5210311,1.0603857,0.41067 ,-0.3616352,-0.25297007,0.97518605,0.85333014,0.16857263,0.040858276,0.09388767,-0.19449936,0.38802677,0.164354,-0.017545663,0.15570965,-0.31904343,0.2223094 4,0.6248201,-0.5483591,-0.36983973,-0.38050756,-1.925645,-1.037537,-0.6157122,-0.53581315,0.2836596,-0.643354,0.07323658,-0.93136156,-0.20392789,-0.72027314, -0.33667037,0.91866046,0.23589604,0.9972664,-0.29671007,0.08811699,0.24376874,0.82748646,-0.604533,-0.67664343,-0.32924688,-0.37375167,0.33761302,-0.19614917 ,-0.21015668,0.46505967,-0.28253073,-1.0112746,1.1360632,0.8825793,-1.0680563,0.0655969,-1.034352,0.5267044,0.91949135,-0.031119794,0.60942674,0.54940313,-0. 3630888,0.44943437,0.66361815,0.073895305,-0.59853613,0.18480797,0.49640504,-0.13335773,-0.66213644,0.08816239,-0.52057326,-0.48232892,-0.2665552,-0.10339267 ,-0.30988455,0.46449667,-0.022207903,-1.6161236,0.27622652,-0.5909109,-1.0504522,0.052266315,-0.66712016,1.038967,-0.21038829,-0.30632204,-0.63056785,-0.0326 83205,0.8322449,0.43663988,0.8234027,-0.69451404,-0.29506078,0.8947272,0.36536238,-0.06769319,-0.21281374,0.1542073,-1.0177101,0.1798313,-0.38755146,0.353291 33,-0.1736927,0.2708998,0.36253256,0.55142975,-0.25388694,0.2749728,1.0570686,0.14571312,0.14165263,-0.18871498,0.2701316,0.6352345,-0.1975502,-1.0767723,-0. 0899109,0.06417123,0.16973273,-1.4618999,0.75780666,-0.37219298,0.34675384,-0.21044762,0.3230924,-0.59562063,0.57655936,-0.24317324,0.4706862,-1.0036217,0.27 595782,-0.18632303,-0.024258574,0.36281094,0.72106606,0.4534661,0.10037945,0.49504414,-0.9208432,-0.8387544,-0.17667033,0.44228357,0.36593667,-0.3061421,-1.2 638136,-1.1484053,0.5236616,0.020920075,0.2590868,-0.017210491,0.48833278,-0.34420222,0.35703135,1.0728164,-0.51129043,0.0902225,-0.42320332,0.19660714,-0.28 81061,-0.15664813,-0.99245757,0.06579208,-1.5574676,0.16405283,0.46488675,-0.15788008,-1.01791,0.84872925,0.035253655,0.40218765,-0.59924084,-0.2960986,-0.27 4478,-0.17835106,0.6479293,-0.42014578,-0.15515843,-0.62578845,0.2247606,1.153755,-0.033114456,-0.8774578,-0.021032173,-0.54359645,-1.0827168,-0.4298837,0.39 979023,-0.031404667,-0.25790605,-0.55900896,0.85690576,-0.23558116,-0.64585954,-0.18867598,-0.016098155,-0.021867584,0.5298315,0.65620464,-0.45029733,-1.0737 212,-0.25292996,-1.8820043,0.78425264,0.049297262,0.033368483,-0.13924618,-0.08540384,0.26575288,0.3641497,-0.5929729,0.012706397,-0.14115371,0.7092089,-0.29 87519,-0.50846523,1.1529989,-0.007935073,-0.39666718,0.66540664,-0.43792737,-0.14657505,0.013367422,0.59577924,-0.31825122,0.3546381,0.11212899,0.5804333,-0. 72722685,-0.58012086,-0.25618848,-0.3021478,0.3090123,0.39833462,-0.1964222,-1.0031091,-0.7377774,-0.37093028,-0.268894,-0.16332497,0.8644577,0.5592706,0.175 96069,-0.28468367,-0.11259709,-0.3321775,0.12905857,-0.4623798,-0.2466813,-0.39571014,0.8273027,0.3286372,-0.42084447,-0.6982525,0.51819134,-0.4211214,-0.450 2746,-0.58659184,0.9362978,-0.24028268,-0.07863556,0.03276802,0.31117234,-0.61217594,0.29426676,0.5394515,0.096639164,-0.17290285,-0.100368135,-1.1184332,0.6 5379685,0.21017732,-0.48588675,-0.42309326,0.78154176,0.11492857,0.9659768,0.85164833,-0.510996,-0.4957692,-1.0045063,0.41195333,-0.25961345,-0.06390433,-0.8 0765647,-0.5750627,-0.004215756,0.6570266,0.021791002,-0.2851547,0.33010367,-1.0438038,0.64198446,-0.3170377,-0.21503314,-0.7744095,0.34140953,-0.123576835,1 .2228137,0.3193732,0.097345576,0.013826016,0.490495,0.16021223,0.3592192,-0.64754117,-1.2467206,0.20728661,-0.040293045,0.18149051,-0.3889212,-1.2567233,-0.2 7512074,-0.8875311,-0.4562278,-0.14274459,-0.7154212,-0.9517362,-0.42942467,0.34255636,-0.25662944,-0.071650766,-0.2570997,0.97032154,0.55209476,0.9512633,-0 .78840256,-0.87641865,-0.31667447,-1.1845096,0.61095214,-0.4934745,-0.090470426,-0.8589016,0.16191132,1.3353379,0.36014295,0.6354017,1.6015769,-0.15028308,-0 .35953638,-0.46233898,0.7056889,0.44098303,-0.2561036,-0.38414526,-0.85254925,-0.35759622,0.32756907,-1.1055855,-0.9486638,-0.75697213,-0.18819816,0.91543293 ,0.046453375,0.8660134,-0.7937197,-0.72757536,-0.8235348,0.8263684,0.84975964,0.6188537,1.0370533,0.8713266,-0.17223643,0.74872315,-0.087729014,0.027644658,- 0.41663802,0.86366785,0.45966265,-0.7807239,0.72492,0.7516153,-1.4882706,-0.7965106,0.44769654,0.04745266,-0.3665682,-1.1761265,-0.16592331,-0.49482864,-0.18 829915,0.079323344,0.5283898,-0.25911674,0.49787715,0.040334962,0.6457638,-0.9161095,-0.52021873,0.3950836,-0.8869649,0.61957175,-0.8694589,-0.14945404,0.331 69168,0.2645687,0.45321828,-0.20752133,-0.00011348105,0.7114366,0.36253646,0.94113743,0.27327093,-0.279275,0.74158365,-0.7394054,-0.9920889,-0.5790354,0.4460 0168,0.6965152,0.055897847,-0.7247457,-0.23232944,1.0741904,-0.103388265,0.3405134,-0.6539511,-0.51377046,-0.7043648,-0.61793834,-0.7072252,-0.34909388,-0.05 701723,0.6294965,-0.30765653,0.03854165,0.032257613,0.8844775,-0.12016908,0.45807433,-0.8181472,0.5738447,-0.08459999,-0.5052286,-0.322389,0.16923045,-0.5340 384,0.82369304,-0.6654957,0.09066754,0.23323251,0.75676244,-0.07526736,0.18891658,-0.58411753,-0.5459881,0.31472534,0.22671345,0.15036865,0.5497431,0.6759999 4,-0.17044614,0.3315073,-0.07908476,0.3493545,-1.3477355,0.56133074,0.6158089,-0.15612105,-0.15391739,-1.6920619,-0.45604506,-0.9460573,0.1832847,-0.9812012, -0.037437357,0.23665366,0.20942298,0.12745716,0.3055677,0.4899028,0.1521983,-0.4412764,0.44380093,-0.24363151,0.049277242,-0.03479184,0.34719813,0.34336445,0 .44446415,-0.2509871,-0.07174216,0.16965394,0.40415984,-0.50963897,-0.4655299,0.59960693,-0.3961361,0.17242691,0.71643007,-0.012265541,0.07691683,1.2442924,0 .22043933,-1.2103993,0.61401594,-0.541842,-0.33357695,0.3074923,0.065326504,-0.27286193,0.6154859,-0.69564784,-0.11709106,-0.1545567,-0.11896704,-0.007217975 ,0.23488984,0.5601741,0.4612949,-0.28685024,-0.01752333,0.09766184,1.3614978,-0.9316589,-0.62082577,-0.17708167,-0.14922938,0.6017379,0.20790131,-0.17358595, 0.51986843,-0.8632079,-0.23630512,0.5615771,0.12942453,-0.55579686,-0.28877118,-0.023886856,0.6346819,0.11919484,0.112735756,-0.2105418,-1.0274605,-0.2215069 7,0.6296189,0.528352,-0.27940798,0.5474754,0.14160539,0.38373166,0.5457794,-0.7958526,-0.53057015,1.2145486,0.12005539,0.9229809,0.11178251,0.35618028,0.8680 126,-0.14047255,-0.022312704,0.6335968,0.22576317,0.63063693,0.077043116,-0.3592758,0.14797379,0.37010187,-0.14920035,-0.303325,-0.68384075,-0.22196415,-0.48 251563,0.085435614,1.0682561,-0.28910154,0.0547357,-0.49188855,0.07103363,0.23165464,0.7919816,-0.31917652,-0.11256474,0.22344519,0.202349,-0.042141877,0.487 33395,-0.6330437,0.18770827,-0.8534354,0.24361372,0.05912281,-0.14594407,-0.3065622,-0.13557081,-1.4080516,0.60802686,0.7874556,-0.8090863,0.5354539,-0.86377 89,-0.2529881,-0.76151496,0.39836842,-0.3637328,0.16363671,0.5599722,-0.24072857,0.09546083,0.831411,0.09562837,0.31388548,0.103111275,1.1427172,0.694476,0.9 3155265,0.64801776,-0.33954978,-0.0988641,0.473648,-0.2811673,-0.3996959,-0.33468047,-0.21153395,0.886874,-0.8678805,-0.10753187,-0.19310957,0.4603335,-0.122 70494,-1.0267254,-0.53114897,0.004987782,-0.7938769,0.40439928,0.4829653,1.5288875,0.6414294,-0.6214873,-0.65656304,0.47653323,0.16301247,-0.12008583,1.03255 62,0.13527338,-0.927417,-0.35502926,-0.17070319,-0.0011159402,0.15795147,-0.3817831,-0.99539477,0.44974712,0.623257,0.032141224,0.20115706,-0.753747,-0.03541 0736,0.317427,0.7414546,-0.41621342,1.4412553,0.088434115,-0.29406205,0.019276256,-0.66831887,0.39378297,-0.15091878,-0.33501017,0.012463322,0.26902023,-0.85 676277,-0.08205583,-0.13279751,0.8540507,-0.07071759,0.67416996,-1.0808998,-0.7537985,-1.1090854,-0.42881688,-0.545489,1.0022873,-0.34716064,-0.3511107,0.611 6534,-1.0079868,3.7511525,0.4171535,0.504542,-0.051603127,-0.071831375,0.44832432,-0.21127303,-0.57512856,-0.19024895,0.23094098,0.16914046,0.21540225,-0.077 53263,0.19773084,0.8750281,0.55822086,-0.46648705,-0.44413725,0.23833762,-0.6311006,-0.5150255,0.014071045,-0.043874096,0.40925947,-0.082470596,0.4262907,1.2 440436,-0.123832524,-0.09172271,-0.42539525,1.0193819,-0.20638897,-0.055872787,-0.12540375,-0.058966316,0.73125196,0.3050278,0.25579217,0.118471175,-0.148029 91,-0.33583203,0.11730125,1.5576597,-0.17712794,-0.2750745,0.11848973,-0.48632467,0.8594597,0.21705948,-0.04919338,0.8793258,-0.6851242,1.2830902,-0.226695,- 1.6696168,-0.4619705,-0.080957085,-0.53974324,-0.77588433,0.103437446,0.015129212,0.2896572,-0.28889287,-0.266523,-0.5023567,-0.0604841,0.57056016,0.5261334, -0.18631883,-0.5122663,-0.055830136,0.56574637,-0.5704402,-0.4263674,0.24019304,0.082071595,-0.31298077,0.30196336,-0.011113114,-0.5608543,0.3951217,-0.26592 582,0.41811758,-0.7411703,0.30873746,0.5664615,-0.98191136,-0.49090472,-1.0648257,0.97027993,0.9559882,-0.019431114,-0.07921166,-0.120092966,-0.13082835}
```

!!!

!!!

Above we used the pgml.embed SQL function to generate an embedding using the `mixedbread-ai/mxbai-embed-large-v1` model. 

The output size of the vector varies per model. This specific model outputs vectors with 1024 dimensions. This means each vector contains 1024 floating point numbers. 

The vector this model outputs is not random. It is designed to capture the semantic meaning of the text. What this really means, is that sentences that are closer together in meaning will be closer together in vector space. 

Let’s look at a more simple example. Assume we have a model called `simple-embedding-model`, and it outputs vectors with 2 dimensions. Let’s embed the following three phrases: `I like Postgres`, `I like SQL`, `Rust is the best`. 

!!! generic

!!! code_block

```postgresql
SELECT pgml.embed('simple-embedding-model', 'I like Postgres') as embedding;

SELECT pgml.embed('simple-embedding-model', 'I like SQL') as embedding;

SELECT pgml.embed('simple-embedding-model', 'Rust is the best') as embedding;
```

!!!

!!! results

```text
embedding for 'I like Postgres'
---------
[0.1, 0.2]

embedding for 'I like SQL'
---------
[0.12, 0.25]

embedding for 'Rust is the best'
---------
[-0.8, -0.9]
```

!!!

!!!

Notice how similar the vectors produced by the text `I like Postgres` and `I like SQL` are compared to `Rust is the best`. 

This is a simple example, but the same idea holds true when translating to real models like `mixedbread-ai/mxbai-embed-large-v1`. 

## What Does it Mean to be Close?

We can use the idea that text that is more similar in meaning will be closer together in the vector space to perform search. 

For instance let’s say that we have the following documents:


!!! generic

!!! code_block

```text
Document1: The pgml.transform function is a PostgreSQL function for calling LLMs in the database.

Document2: I think tomatos are incredible on burgers.
```

!!!

!!!

And a user is looking for the answer to the question: `What is the pgml.transform function?`. If we embed the user query and all of the documents using a model like `mixedbread-ai/mxbai-embed-large-v1`, we can compare the query embedding to all of the document embeddings, and select the document that has the closest embedding as the answer. 

These are big embeddings, and we can’t simply eyeball which one is the closest. How do we actually measure the similarity / distance between different vectors? There are four popular methods for measuring the distance between vectors available in PostgresML:
- L2 distance
- (negative) inner product
- cosine distance
- L1 distance

For most use cases we recommend using the cosine distance as defined by the formula:

INSERT IMAGE

Where A and B are two vectors. 

This is a somewhat confusing formula but lucky for us pgvector provides an operator that computes this for us.

!!! generic

!!! code_block

```postgresql
SELECT '[1,2,3]'::vector <=> '[2,3,4]'::vector;
```

!!!

!!! results

```text
   cosine_distance    
----------------------
 0.007416666029069763
```

!!!

!!!

The other distance functions have similar formulas and also provide convenient operators to use. It may be worth testing the other operators and seeing which performs better for your use case. For more information on the other distance functions see our guide on [embeddings](https://postgresml.org/docs/guides/embeddings/vector-similarity).

Back to our search example outlined above, we can compute the cosine distance between our query embedding and our documents.

!!! generic

!!! code_block

```postgresql
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?')::vector <=> pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'The pgml.transform function is a PostgreSQL function for calling LLMs in the database.')::vector as cosine_distance;
SELECT pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?')::vector <=> pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'I think tomatos are incredible on burgers.')::vector as cosine_distance;
```

!!!

!!! results

```text
  cosine_distance   
--------------------
 0.1114425936213167

  cosine_distance   
--------------------
 0.7383001059221699
```

!!!

!!!

Notice that the cosine distance between `What is the pgml.transform function?` and `The pgml.transform function is a PostgreSQL function for calling LLMs in the database.` is much smaller than the cosine distance between `What is the pgml.transform function?` and `I think tomatos are incredible on burgers.`.

## Making it Fast!

It is inefficient to compute the embeddings for our documents for every search request. Instead, we want to embed our documents once, and search against our stored embeddings. 

We can store our embedding vectors with the vector type given by pgvector. 

!!! generic

!!! code_block

```postgresql
CREATE TABLE text_and_embeddings (
    id SERIAL PRIMARY KEY, 
    text text, 
    embedding vector (1024)
);
INSERT INTO text_and_embeddings(text, embedding) 
VALUES 
  ('The pgml.transform function is a PostgreSQL function for calling LLMs in the database.', pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'The pgml.transform function is a PostgreSQL function for calling LLMs in the database.')),
  ('I think tomatos are incredible on burgers.', pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'I think tomatos are incredible on burgers.'))
;
```

!!!

!!!

We can search this table using the following query: 

!!! generic

!!! code_block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

```
                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
```

!!!

!!!

This query is fast for now, but as the table scales it will greatly slow down because we have not indexed the vector column. 


!!! generic

!!! code_block time="10.493 ms"

```postgresql
INSERT INTO text_and_embeddings (text, embedding) 
SELECT md5(random()::text), pgml.embed('mixedbread-ai/mxbai-embed-large-v1', md5(random()::text)) 
FROM generate_series(1, 10000);

WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

```
                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
```

!!!

!!!

This somewhat less than ideal performance can be fixed by indexing the vector column. There are two types of indexes available in pgvector: IVFFlat and HNSW.

IVFFlat indexes clusters the table into sublists, and when searching, only searches over a fixed number of the sublists. For example in the case above, if we were to add an IVFFlat index with 10 lists:

!!! generic

!!! code_block time="10.493 ms"

```postgresql
CREATE INDEX ON text_and_embeddings USING ivfflat (embedding vector_cosine_ops) WITH (lists = 10);
```

!!!

!!!

Now let's try searching again.

!!! generic

!!! code_block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

```
                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
```

!!!

!!!

We can see it is about a 10x speedup because we are only searching over 1/10th of the original vectors. 

HNSW indexes are a bit more complicated. It is essentially a graph with edges linked by proximity in the vector space. For more information you can check out this [writeup](https://www.pinecone.io/learn/series/faiss/hnsw/). 

HNSW indexes typically have better and faster recall but require more compute when inserting. We recommend using HNSW indexes for most use cases.

!!! generic

!!! code_block time="10.493 ms"

```postgresql
DROP index text_and_embeddings_embedding_idx;

CREATE INDEX ON text_and_embeddings USING hnsw (embedding vector_cosine_ops);
```

!!!

!!!

Now let's try searching again.

!!! generic

!!! code_block time="10.493 ms"

```postgresql
WITH embedded_query AS (
    SELECT
        pgml.embed('mixedbread-ai/mxbai-embed-large-v1', 'What is the pgml.transform function?', '{"prompt": "Represent this sentence for searching relevant passages: "}')::vector embedding
)
SELECT
    text,
    (
        SELECT
            embedding
        FROM embedded_query) <=> text_and_embeddings.embedding cosine_distance
FROM
  text_and_embeddings
ORDER BY
    text_and_embeddings.embedding <=> (
        SELECT
            embedding
        FROM embedded_query)
LIMIT 1;
```

!!!

!!! results

```
                                          text                                          |   cosine_distance   
----------------------------------------------------------------------------------------+---------------------
 The pgml.transform function is a PostgreSQL function for calling LLMs in the database. | 0.13467974993681486
```

!!!

!!!

That was even faster!
