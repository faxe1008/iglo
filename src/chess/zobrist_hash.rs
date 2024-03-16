use super::{
    board::{CastlingRights, ChessPiece, PieceColor},
    square::Square,
};
use serde::{Deserialize, Serialize};

const ZHASH_TABLE: [u64; PieceColor::PIECE_COLOR_COUNT
    * ChessPiece::PIECE_TYPE_COUNT
    * Square::NUM as usize] = [
    0x149e345028558a4f,
    0x44755cbfee0f8572,
    0x731eca45667cd26a,
    0xcb2e1b5403fc5d50,
    0xaacb0ce08fdfdce4,
    0x3266bd39d53d8707,
    0xaa309a613c6df786,
    0x97d99fa45b4595b8,
    0x8f5bef933b997d7b,
    0xaba040d44a1528af,
    0x929f71caa2596cbb,
    0x21e24c60298629e2,
    0xcc12914660ad8ffe,
    0xcafe38c1c44a9c26,
    0x90ed5193991ce49c,
    0xfa73ddd2f4ab1e93,
    0xa22110133ed11761,
    0xc64e2192785c9b89,
    0x22ec218391b4fb14,
    0xd8d1b88f85cd065c,
    0x90b8779f8c6d736d,
    0x2ad03bea2a1714f9,
    0xd669325edef08e98,
    0x3b7aa5580fef3b05,
    0xacab8c6f427908f5,
    0x5f171173fa11de25,
    0x00b4faf8b3e46af8,
    0x0c2158d16ed39913,
    0xacb09ba3c34fe394,
    0x2f55ec504ce6b601,
    0x2218083c175d244f,
    0x3deee78fcfaa0d20,
    0x9383ddd1b3f65d8d,
    0xf5b7e27f0cc51d3b,
    0xbc7b622248ce5626,
    0x7a6fff82a258da2c,
    0x1ac7957ded7feb24,
    0x89e758333a360cce,
    0xcd9107a75cea9bc1,
    0x171367d8c900c8a2,
    0x2a746103e7878e48,
    0x7d9e749c7469aa37,
    0x95debbeb97541608,
    0x4f38a1e45d8fa2be,
    0x57b9be2e84ab5920,
    0xe9b6b0d045140c11,
    0x0a437f84e0868010,
    0xf5028a9cdf23d9fb,
    0xd5e3369e616254fc,
    0x7fc3ab20ff8eddf1,
    0x96e98f6ce7d3d6ce,
    0x264fd2e317c97af6,
    0xab772c11632dbd9f,
    0x7872f6cedfc514e3,
    0x887dbc2eba6a5f11,
    0x187df065e27fc377,
    0x17719ae8f23c2caf,
    0x1e15e23279e02c0c,
    0x133975f36119bb81,
    0x8c0c153aca06cfa6,
    0x9b733c77bbdc47b5,
    0xec5047d35e5d803c,
    0x8ec60d3bf664c7dc,
    0x9b6c53b8b082e856,
    0x871d06fec7fc4c21,
    0x7db625d4354ee0fb,
    0x024c8fc8ba113a7d,
    0xc0a86ad21f9f5a88,
    0xdce8eb36e870e30a,
    0x7a072c940c310d2e,
    0x0a4bc9533e16ebb0,
    0xde0dfeb187b42eac,
    0x417657fb6a68d308,
    0x54e46811eedeab8b,
    0x73f8295d1968072c,
    0xdebcff2088434806,
    0xe63f1bd418b02809,
    0xc762529c45bf8f6a,
    0xf29ca48f9b789307,
    0x81241b3ff8fddf99,
    0xda582bdd76b55b1d,
    0x60d5f6a1227546cc,
    0xf214ae04baf2e0ff,
    0x2b9bd56ec9d7dab8,
    0xbcba851a3312aa0f,
    0xb928b4ed99034f51,
    0xc6773ea873096cbc,
    0x0a65b9e8fe52796a,
    0x7b6908fd9ac8d254,
    0x88e39f3c2e0d721d,
    0x610eea669fa18068,
    0x9fc996c7534ad354,
    0x18ecd089954993dc,
    0x5be304e142f0702b,
    0xaed5c5fe6c30ba9a,
    0xb7defd0b386b47e2,
    0xb0de077a3295c180,
    0x2b6e3963f7e7bae6,
    0xeba4c5b480d17a22,
    0xc3006e316dd26c3c,
    0x641485f4923127f5,
    0x22b6a87986f7c7bb,
    0x73b1e547bb670cee,
    0xe4665d3db102ddd6,
    0x7c1e9c0b9d9a6ea2,
    0x533a2a990c342f15,
    0x730af351bd8454e7,
    0x597c736fc047d018,
    0xb9e63f928d838f48,
    0x11731795da69e08b,
    0x9f9b4c5021161687,
    0xaa8d8e05630d69ad,
    0xf0d3616c86c28388,
    0x1843830dceb5cdd6,
    0xb15c55d6d3705529,
    0x70c4cedfc72f34aa,
    0xb66773366f7ee064,
    0x134c7f2a3f0a03c5,
    0x51b7cf4a0b843078,
    0x643c4773dffacb76,
    0x6ec7b0a55a1d4e27,
    0xa1e4454e646af0b8,
    0x73a03a3261ac65c3,
    0x4c453d409a6fe240,
    0x221b4e1561b0fbcf,
    0x6a99afa0956f77bf,
    0x2bf907ffa0d0c438,
    0x0ed900bc13a8e193,
    0x9ead926493335f9d,
    0x4b3cdac310c84ed2,
    0xb83268fee19f9e84,
    0x06bb78bf2575c272,
    0xe1fef6df53f21db5,
    0x061d7d12fcfccdfa,
    0x5c0d6c1c571f5b95,
    0x208354d07c758b3a,
    0xce38fbbd411295a1,
    0xe65cc6ffdf00117b,
    0x8b0533c693cf36ca,
    0x70acfbebb05430da,
    0xbf5b62b2f2c321b0,
    0x28565ac765b5ea00,
    0x3b3c1517c3965e90,
    0xfd6be76daf51f90f,
    0xeaa5b84413392179,
    0xb9b4afd2f8b75839,
    0xd36ab433f87124ba,
    0x09fbb04c17b094d1,
    0x91935d7b602104cd,
    0x21f70bebeb98cdcb,
    0xbe8174a0043b8532,
    0xc602180299efb908,
    0x2f4b0394f8264ad0,
    0x8bd07da8350fbf3b,
    0x4c63c8a95766fd15,
    0x21ece7365c9f1637,
    0x45480a0c9ec42700,
    0x969377c12c357dd7,
    0x6267a1eab3759331,
    0x1cfbaaf038c9cd03,
    0xac0907e08d0e714e,
    0x5350bbb145ff9d1d,
    0x6661be3586f9230c,
    0xe1dc0f23f9b97911,
    0xc6f8cba17cddeec2,
    0x9a8571f971e64790,
    0x7c8f98c1b77cbe46,
    0x0661c1e2cf56a7f2,
    0xc755404fa9281490,
    0x7bf784879545c2c2,
    0xeda443cfc8b216c3,
    0x191a2cc9b58bae1c,
    0xa90c7bec6fa90710,
    0x6c0259f52c608c95,
    0x8c93eaf8f41035bc,
    0x6b4cde809275e56e,
    0xda9a0a6a15c02c71,
    0xff0776922cc09df2,
    0x21cfb8c14a0b91f8,
    0x9350cf3fbecd181d,
    0x3d45a5086a9b0af1,
    0x64146efa592f7e8b,
    0xec14b41557841e3e,
    0x8d68c73dbdaede1a,
    0x30f18b820e521716,
    0x01616595b8dccadb,
    0x38761fc75f53127f,
    0xe0d139de062db813,
    0x027df4877ef3b1c6,
    0x5c016e613f6b59fe,
    0x9cf2082c3afc6713,
    0xaf2a220696d933d3,
    0xfa69edee2686dac0,
    0x4f072eacbe7d937d,
    0xb9a35ed5855ad91b,
    0xf241f250b4ed79ce,
    0xe926fd0246bfe1ab,
    0x37d51733c9761128,
    0x959ee96c6c43cbcb,
    0xce03c93f25c1998a,
    0x96f5e945453979be,
    0x8f190ab628519b37,
    0x71f6ab970c13f62a,
    0x59183fdb9c549427,
    0x0c1b9ce808b1fdff,
    0xad930b8b9376af8d,
    0xdf1d7c2eccec7399,
    0x02e512dff01a5c10,
    0x77a4017837d5ab80,
    0xd52f1ec63b8858bb,
    0x9a55289f6c1accfa,
    0xe629c87cc0a21772,
    0xd8faca9cb4c40dab,
    0x91d9effc43817301,
    0xfcf53c7b34a3c902,
    0x760b66855efd4566,
    0x48ecea44436e28f1,
    0x85c16e8524411040,
    0x9ccead927c221f4e,
    0x7923b496b5b8a57e,
    0x91d906e8c25279e4,
    0x240831f9cacc2a04,
    0xb016e47bd6936426,
    0xde49924ee10c247c,
    0x42b3c3ed5f105560,
    0x304a94677f8dc462,
    0xd3ac11019e60038e,
    0xd0de8a634aed23b1,
    0xfac6a5e7f338843b,
    0x0715dceda15d7d4c,
    0xbf12c0d7a824bd8f,
    0xccbe480176697742,
    0x3f0bb754b0fd5e3e,
    0x57035377096fd111,
    0xe2e9e8571eeae060,
    0x52543c7283acebb5,
    0xd3343e8bad8de483,
    0x6e7a242cbbb9f98d,
    0x44c06f0a99577aef,
    0xc35e8171cc7df635,
    0x384967a1cd45c7ba,
    0x57c6a21831e8d227,
    0x4b35fabc2854467d,
    0x9904361e62a7debd,
    0xe973ea05994b4391,
    0xf7b836c3126f5e8d,
    0x0a33bf459305be43,
    0x9617a35f6b0eb247,
    0xc1e30f0c26a781ab,
    0x1e60e8427743189f,
    0xb4d5433bc4d80d11,
    0xfb80ca2608b1663b,
    0x2bb35a6d00797966,
    0xab2ff4303f48960d,
    0x9116e7b4195e7782,
    0x0be736651e8ba4de,
    0xa67a1ae29c165376,
    0xebee8ac37d3fc224,
    0x3c2dc67651059dd8,
    0xeb304fafe16ddb15,
    0x85727e29500a8354,
    0x26a5859a39747cc1,
    0x031c20938bd5e073,
    0xad2d893f2ea4357a,
    0xb52a6bfd706a84dc,
    0xa43d61a1f8dd3d3f,
    0x350160e030927798,
    0xb8832b85c150c524,
    0xa3387519f9c6ee45,
    0x56ef23c52bab7c5e,
    0x676ef2c22a76947b,
    0xda644f689e26d5ee,
    0xc9de8ce394f5cba3,
    0x79c30df9b8692ca6,
    0x86c98789a941e161,
    0xa562baf608e9c80c,
    0xa792c82b428f73eb,
    0xe6e619f038c539ef,
    0xbeb78d34dcaccc14,
    0x32bc684840c46de1,
    0x9d151dc62ea57043,
    0x22a5554734837b19,
    0xcc958f40b16b00ee,
    0xcaa1d64bd4d7980f,
    0x2c1c79a80002a0e0,
    0xbfd0d1bd017e59d8,
    0x5d2cfe385bafbf30,
    0x41bcfae9c365b7e7,
    0x590c290b024a8ebd,
    0xc770a1696553e202,
    0x57225be4fb511a8b,
    0xa1dbba2375303713,
    0xd6e0967b504b265e,
    0x0749ba3a9c2ca8a4,
    0x93aba8edb406519d,
    0xf54468531f3589ba,
    0x1cac68aa07078793,
    0x209a5413a2321a8c,
    0x241eb0557fea0ba1,
    0xe671b6ae258cced3,
    0xe452e5ddddfbbb8c,
    0x7054836d840bcebe,
    0x164de5d9244b6083,
    0x8d4ecc6c1fe61b23,
    0x2d43f62ff76d65bf,
    0x8173c6f1e2f2de99,
    0x780489d38adc09f7,
    0x2ebb5c0b0b17d30f,
    0x9eff4699819cea70,
    0x76469358c00ba179,
    0xe1a3304d584364d8,
    0xc4fe831409ee993b,
    0xe0be898d73113d89,
    0xbe43a87fe7518e7d,
    0x1f9a0d367597ad2f,
    0x67838dcb830ec7ef,
    0x92417b7e4804ae8f,
    0xaf0b2c8f5cd2e57d,
    0x4b38660e548b330f,
    0xd1f254e1c55f85ce,
    0x1def3555ed9bb75b,
    0xcaf4b76ba65acaa3,
    0xe8fa91d0f08f473a,
    0xcf1234dd56919fc0,
    0xf5a65d6800da0365,
    0x27a8d86d5990ee5e,
    0x571e492362504d53,
    0xbcc5314e4df3c020,
    0xd02593710df19e3f,
    0xe4366265b66e9587,
    0xb34a46842237b86b,
    0x28cbc60e635b8946,
    0x3df17590cd06f17c,
    0xb6a2d6544c115490,
    0x36887a496a6d15ce,
    0x135019427c91a445,
    0xb034991b02211b5e,
    0xbdf9a8580604310e,
    0x4e474638b624a083,
    0x481adb74061bd75f,
    0xccff9f7802991e8a,
    0x62433f41334cafa8,
    0xe06b8739222f7dfa,
    0xc2ffeb235625dda6,
    0x5e3f353a85721851,
    0xe555355c2d9d1ae6,
    0x50dcd86f16e15ad1,
    0x66e35f187448ad54,
    0xeef3e542ddbeeab5,
    0x36e9980e31570da0,
    0xfae14194843bfe05,
    0x9d3a05b52b8d3082,
    0x5a65892d1bb4207e,
    0xe0421eec218f43f5,
    0x9996d4e367a79d65,
    0xecb62a49d7d27b47,
    0x30ac54056ba95d5c,
    0xe814763e7c4f831d,
    0x7edd1851cf9f5f70,
    0xd12249d9fa17d23d,
    0x32a5420f33d6fb64,
    0xed8b50011d54016d,
    0x1e98d690538641d0,
    0xa1f09ce577ffda41,
    0x7be87173df908a1c,
    0x7ad7091fa19cb42c,
    0x19973f1ad771ff88,
    0x520ab5cebd5241df,
    0x01e7343064e6e3c6,
    0xaed095c8cb3fc0cd,
    0x1ac7e75aa9d70a80,
    0x41580efc76feba66,
    0xc15475430c0a5ca5,
    0xb967b6ee97b1cdf0,
    0x82a18ae280ddc873,
    0x14e8c750bb325dc2,
    0x6ae5559a977aa4e2,
    0x8575109d9efa91b7,
    0x828ba1ffd067c80c,
    0xf8835137dbd93f02,
    0xa18314ab87116a37,
    0xd8854707639420ce,
    0x0902398069d297d9,
    0x159d325822ad5709,
    0xb09eb5870da422a2,
    0x070d32b4e5f0f109,
    0xb496365cda9db7bb,
    0xd85a00c57d887c9e,
    0xc8811ebd1751e7fe,
    0x0eea6d9e8bb4fa3a,
    0x54cd5c60507dbe9e,
    0x1e024a147f02fd1d,
    0xee6f406313e0906a,
    0xc1e9dc701d6ff649,
    0xeb70486f79f44892,
    0x5b0182f3653fb238,
    0xef4b403e6a4d0f2a,
    0xdb4ae02ac60dd851,
    0x83b917f33f0fb531,
    0xba40532723fd404b,
    0x6abf7b69a21a053d,
    0x6a7125d46020179e,
    0x4a9a2ccd56f72ca3,
    0xbe4cca22ab42bc33,
    0xdf9cd945de98382c,
    0xeabbc0202934ae1d,
    0xfe59786acbda8807,
    0xf61e7b8531a9bd9c,
    0xb8237b6fc48131f4,
    0x3b87d221d02bfc61,
    0xba366ed23ac8d3b7,
    0x6f4865c3e117ed34,
    0x0e0d57be1d488ee5,
    0xe47446bcf9b724e9,
    0x7a222e325271f2bd,
    0xf3b3086e1b0a0ae1,
    0x274a9ba6f6e68737,
    0xe53bcde61479426d,
    0x8504488828d0febf,
    0x718af685e74a1f1a,
    0xbe198b98c9b8e734,
    0xb94ff28f323a707a,
    0xf0172d8e9d1f0b37,
    0x6fa3665947005c31,
    0xef0468fcb1dc99d3,
    0x66f7807a0f0e868d,
    0x01cc6cc1703643cd,
    0x4dae3b6936f9cc26,
    0x2580ae63eb47edb8,
    0x4cdae47b84238ecb,
    0x78f54ac5b8568560,
    0x9aae282279d70edc,
    0xabc0f10f7608cbb9,
    0xeb9d45e59c665810,
    0xe3f19ebc71f02d46,
    0xd708bce7c0caabda,
    0x47a3e459ed5bda54,
    0x58f26b81ed893314,
    0x2dbb4c56d74c21b3,
    0x35ce85093edcff2b,
    0xfaca5baac06127fe,
    0x024ad9ce4bf5d3f5,
    0x84399268041e850a,
    0x3ab9a5cbf9725f82,
    0x9474c50719d106f0,
    0x69536b4a705d8ee4,
    0xf143a37f72dce09b,
    0xf2357d388d09802a,
    0xab0cadf0d5bffe63,
    0xee6073c585a00cbc,
    0x2741e27431b9ab1b,
    0xba4c6dff6f61eb52,
    0xcf3e58b187014f91,
    0xfc0a1af9442169ed,
    0x03632a5fd89650eb,
    0x52b536bf60f1508b,
    0xaaecfee8f3719c48,
    0xc79b13f84d9837fc,
    0xec49092962ea54f1,
    0x86bb48696d97f0dd,
    0x9760a1fc0142b560,
    0x93fd23fd2e07a6db,
    0xef00b8a1fb44a46f,
    0xba38663a1c3d23da,
    0xc0a5f24d23859d8d,
    0x8a858b002b7346b5,
    0x8506ba6d30fdb3a0,
    0xc9b00fec6398e026,
    0x4b363755b6e06a27,
    0x85603f935ed3e865,
    0x67ef520cccf41dc0,
    0xd506a62f332a20f5,
    0x0c7961caa7ed7f82,
    0x883a4e55bd97cb41,
    0xd318948ee8dfb461,
    0xe78b2e387513d4de,
    0x68454135e953758f,
    0xb0498827ecbe7104,
    0x4dd9b6fed05b2862,
    0xe13443eb2ce36925,
    0x6bc20aa9c3d8625e,
    0x9791714866a6a75e,
    0x6df8303cfdad5f50,
    0xca44c923326a23f7,
    0xb503681d1acfda10,
    0xb564e70142489288,
    0xddcdcfbeb37cc428,
    0x7d205c9eccab449a,
    0xd865922a34fa51d2,
    0x159fdf6529084fd8,
    0x318e259c067acd15,
    0xeeb7fe85345da377,
    0xcb77f53b6caaa6ae,
    0x1e7d4c8039b7591d,
    0x9802a3c41c5e4c31,
    0x55f4bf0c8b12fe64,
    0x0ac32b778b4d584b,
    0xbd6c043b3bc3d1b5,
    0x8a3331624585db04,
    0x58b4505b7df8207e,
    0x8878b4c44abd8d01,
    0xa3083b5712253f33,
    0xd99615ffdb228157,
    0xfa447076d902f9c8,
    0xf1227734c19ddbe1,
    0x2814032b03a06bbc,
    0x7361c46a423b9317,
    0x3e6ab6cff7b7d264,
    0x299802dd302cdcdc,
    0xff85f8e95b1cd170,
    0x620efe2163fa485f,
    0x7743bbcc3673cc07,
    0x4bfaf451540d4b69,
    0x591de20508e9455c,
    0xa5452bc155fc2c8e,
    0x40d106aaf9cea61d,
    0x54cb40f8bf5b92ba,
    0x3fca4f37972338d0,
    0xc9f28126599f07d9,
    0x647e5f6dce529860,
    0xae16d093f7e828d4,
    0xf5611f22152d54ba,
    0xacd63670088faaf4,
    0x5667a6a907ee5df3,
    0x437f5a4f07e5d537,
    0x52901b8644e1de56,
    0x3c80de4356b025b8,
    0x205d0ebaf40d634a,
    0x0f3ea1209b37f2d9,
    0xfc2581825af035c7,
    0xb7346693442b7c44,
    0x3c6549b81439a00f,
    0xf9a7ed0da3ed027e,
    0xb7755a6e70edb49d,
    0x5c3a3c97cd70aed6,
    0x02e69f701d2a1d65,
    0x62ba16cc7cc53719,
    0xf567e072e2675ce8,
    0x717d057431b28168,
    0x71a2b5631d65da08,
    0x5d061d5133fe48e8,
    0xf7604436990a92e6,
    0x121738b4dc8eb099,
    0x7e24ac98636c196b,
    0x84219ad271e94dbd,
    0x188daad4e07cbdba,
    0x06f05f57bf45e1e7,
    0x1817212fac8000ed,
    0x68343ddcfc4c3afe,
    0x728289edc60718d7,
    0xd5ab74438acccdd9,
    0x3fe658f1aca905ce,
    0x6e35856beafe1e4f,
    0x640c53429a319234,
    0x707d26948889305e,
    0x65bef5f088034bfd,
    0xbeb5fbcea73f4472,
    0x3ff03aee82fdb024,
    0x549b3c19450cb249,
    0x80c4f0e6370f47c8,
    0x2dcd66f4e00cab84,
    0xc21e8fd195bc14c5,
    0xaa5cc737c0355135,
    0xdc6a96d1855c2f03,
    0x91b656ab5ff34e0b,
    0x849d8075609b55bd,
    0x19db83dc4a293cfb,
    0x8825c352a59a3f32,
    0x98f80a9de140e3dd,
    0xa3c7e03ea88abed7,
    0x1a071e191dfe020e,
    0xd35c634ca27b8c6e,
    0x52013293538062ac,
    0x05d957d50f99690a,
    0x5acf2f8f42d9de41,
    0x23c6009d925f7a6b,
    0xa770110c52f53637,
    0x6f9cc6d9f16f52fd,
    0x954e2e603da06c13,
    0x70bdba9ca246506b,
    0x31a29b039dfb0d38,
    0x927272338748d837,
    0xe20776c21154d880,
    0x307406a889f3ea45,
    0x89f5bcbfcad11726,
    0x6c8efebd25b96d3e,
    0x680e85f86b959691,
    0x545975f53f1e2a27,
    0x14548a5533c9478e,
    0x9a0d42926f0f9759,
    0x9680968aa1855c8d,
    0x10d3dcaf6eaaf996,
    0x785c748d8ead2a93,
    0x1187497019a4cb5c,
    0x050c460b32cf5332,
    0xf0468742f41ccfdf,
    0x0dfbac38bd0a2bce,
    0xb11d4d0942f73ea6,
    0xa490c752afabce95,
    0x12b3548806613ac3,
    0x14f5b65881ca02f5,
    0xa431eb588fb6f036,
    0x5e31223ca0de804e,
    0xfeeb40f04e5a344c,
    0x137c5310c5737af8,
    0x81fd6f3b516d2c91,
    0x4a0495e9f2ba6313,
    0x153573253fdabfa1,
    0x5aec020396030e67,
    0xc18c7068512538d0,
    0x299bb21b4346d63f,
    0x6d924a115a122aaa,
    0xc1492f5c65ec6556,
    0x212b1dd1dd0845df,
    0xe829f17d694b10d3,
    0x9a1abf52debdb57d,
    0xf140b58c6279a9b5,
    0xbeb510c54582bcc3,
    0x0c2d5c5a58f0a584,
    0x045e68678d1f14b6,
    0xa3fcce87aee0990e,
    0xfb78fd19da5c3030,
    0x5203af37b7d8ad0b,
    0x32f9b8a3f9c8500b,
    0x1bfebf170accb249,
    0x3bc4610bd12ec90f,
    0x4508e9bd4b9e6b9e,
    0xd085f291737a2804,
    0x387acb9a4b0e002a,
    0x00512b051243b7f6,
    0x408238bb0c7d1ee1,
    0x71855c2ad2c342ff,
    0xfe90fd2d52a6ad21,
    0x9548e39069cb0fd0,
    0x7d918e5e5ac2c9dd,
    0x8776e91008e668f6,
    0x5f6206b284da7edd,
    0xaf8be7c5e15549c6,
    0x659ad7cf66fae245,
    0x1182091495efb7fb,
    0xd376884f33149973,
    0x41a82b8ba09e023e,
    0xc99eae34b7a5d624,
    0xb1fc8deee51063e2,
    0x786a27ad08e0541d,
    0x772e8236ba2c839e,
    0x2ea9492df6b4fbe3,
    0x41defb9065f3461e,
    0xdac701685eade548,
    0x2d4642381cc599e6,
    0xa206a248d4c7ad4d,
    0x8eb82b7171e1f9a5,
    0x53a88d96602ea9bf,
    0x9aff521e26cd0218,
    0x731bf45abff32ff5,
    0x8598401b25ba17e2,
    0x99727d3383f112af,
    0x543206c1513ab5a9,
    0xdf629fad7615aaac,
    0x122ce3946db532e4,
    0xb6909dd675ee03c7,
    0x021597c3841867d9,
    0xedc0af49c768f441,
    0x7f3d1a71f356ccfa,
    0x4867c50b5c8e95c5,
    0xac78a5803d19d6b9,
    0x01ebdd799010f26b,
    0x485d9e355558f630,
    0x611f993cb0a6381a,
    0xf47c274fe310a915,
    0xe78d06cd42b37d03,
    0x00e7bf40a98116fa,
    0x2544c00efa3f1c55,
    0x02ffb7b6d29a8f3c,
    0xa7ab7acd8ab38269,
    0x2d94e3db5bfff5d0,
    0x113e727f183db65f,
    0x0ef62626db24f59e,
    0x567c798e475a9f44,
    0xeb4e9956607c2a04,
    0x887c82b9dace7235,
    0x00cb3aa74598818c,
    0x1065882ea570c028,
    0xf30896f7a7c6c8e9,
    0x04e421d4ce0ce336,
    0x2249dc5bc84689ed,
    0x287b72e9fa22dacf,
    0x5fbde72e644f69cd,
    0x5963f1a92e9084f7,
    0xfe5597bb7ff54e45,
    0xf99376e2bf05eedf,
    0x228a72eb99dbb184,
    0xedd83d2c17a2c296,
    0x0ba6b2ffa53db5e1,
    0xe0de9ef296a035f0,
    0x0bb836fb3e1076ee,
    0x99b346df3847c827,
    0x903e2fbb569e197a,
    0xf14953a0e47db122,
    0x42abb498f498cb1b,
    0x0cc6481caad5cc0b,
    0x2298554ef202c7ca,
    0x2759f4ae8a5733ff,
    0x38846a77c723d2dd,
    0xfd716fbe04b1bff2,
    0xda99f2304329f3a7,
    0x1499bd131dbf6517,
    0x48e44d0e95be4001,
    0x012786918190d412,
    0x984ce654220ed362,
    0x0334c355f12ecc33,
    0x1ac56ac9f0dbe9b4,
    0xafc363c9b4fb02aa,
    0x571e5ccbaea5066a,
    0xb65e209c89322980,
    0xd69f9e72a8b40dff,
    0x35a7be897387bc90,
    0x3f804cf7d0346f50,
    0xb0c26049a656f6f0,
    0x817fe6646e5db25a,
    0x4b34d5d1c570b8a3,
    0x36ca654c5adf1d93,
    0x6bd8cbcc9c87e59f,
    0x4c14021f01269d1e,
    0x26a36502e30cca9e,
    0xdf38ee56be84b59a,
    0x0d718bff42010f89,
    0x8558795a13c5213c,
    0x17e32b959d82bd74,
    0x6a1e5d5b05fba733,
    0x5a6739731689dd5d,
    0xc1130d355ea376e3,
    0x1fdea019b6c3ed43,
    0xbd1c1b696c72b25b,
    0x23ca36439c61f454,
    0xf29223b64ea13a28,
    0x8388a066b46a866f,
    0x16100f85209dd6ba,
    0x755cee5d1c152f7f,
    0x6cca7af31b285d46,
    0x8d4df754d4b33610,
    0x9a55d66e9c3bd495,
    0xc2df633f43776973,
    0x19454d83793c92d4,
    0x8aa05fd4cc6a1aca,
    0x71acc4f0a260c400,
    0xd0fadbc52a7b0e3f,
    0xd028fbc98f2b9941,
    0xc2b0c2909b76859b,
    0xb5f9eccb3b4b78d6,
    0xc36547dae87d1af0,
    0x7c7060dd729b7072,
    0xcbab95b6b1742525,
    0x1361f8547a5e1b4e,
    0x44164736bb0691b2,
    0x9655ce51a6bf85d3,
    0xa9d987840c74ef4c,
    0x8e1cf2eacd53a68d,
    0x45c0989ceb8067db,
    0x10bf0343695e7a88,
    0xc03787dd382b6c2c,
    0x91f7b572549c46cb,
    0x0be9e1a6bf7782a7,
    0x9909967bd79e3f93,
    0x18dd500956d2795e,
    0x863b678e47d9603a,
    0xfa9a5a174e54c598,
    0xc2fc220af930af68,
];

const EN_PASSANT_TABLE: [u64; Square::NUM as usize] = [
    0xd3581c32769ac1ce,
    0xdb3c6ef2ab045e6f,
    0xdd97d3c2af5ebf19,
    0x25249302971c6e3d,
    0xf2e9b7b9b11abf73,
    0x09cbbc8e59394dcf,
    0x1a39ae1d396dc08c,
    0x5618d1ee28191bfd,
    0x0a2bc485303e3f42,
    0x526cbe945e1a5ef7,
    0x6d347aae372dc90c,
    0x50e7a32d4cd637b9,
    0x4253dd046597562b,
    0xfdbc73d9f60571b6,
    0x1923cd65c44cbc54,
    0x8e9c9f1fa6306742,
    0x11bd2ce88eab7496,
    0x5c220a13edba582f,
    0x32d6fad7445ebbcd,
    0x795ecb68f0302f54,
    0x64f2f11c161a3763,
    0x7ff30469cc29f480,
    0x992927e2121dcb92,
    0x471f9329c69331b6,
    0xa6f90c87b94c695d,
    0xc898dba7e265105c,
    0x9d5ff40cb374a03d,
    0xd1c8ab7d4d4d81dd,
    0x1c71251690593287,
    0x0a66de8cbb9cb9c0,
    0xe4efe83f55999066,
    0x8f26e71485649158,
    0xb6535e720bf7dedd,
    0xb0384b11055745e3,
    0xb2f365562f7bb153,
    0x7e639b96c12b751e,
    0x8cf8de1f85cdac3a,
    0x6ead6aaca8ae7a6f,
    0x7335280276058355,
    0x199659360dd81648,
    0x4f3e19170a322516,
    0xa5829877d96057be,
    0x7b50f9cb0c9c4984,
    0x5f31e6c9c2a6d2cc,
    0x7ca6ddf94beac5db,
    0x5407b7479889abff,
    0x7a5d233feacced4d,
    0xb6f10bed554b21e5,
    0xb9c021c97f3628f1,
    0x1d1f50c4911691e1,
    0x487c4ad095e790c3,
    0x77cd934aa2747b48,
    0x4088a00ade404335,
    0x00cc3cd582516054,
    0xb2d66a313cf4a505,
    0xcee95b2187f4700f,
    0x8b1302c6e6a5373d,
    0x773f784c96fa6156,
    0x77e81a84b839c0b9,
    0xccf0b80c26c4da5a,
    0x2882340c882b3c95,
    0xd68488c8a6aba401,
    0xcc19d60724c2afe5,
    0x4dff6125f154ff81,
];

const CASTLING_HASHES: [u64; 16] = [
    0x40dd164d9f66630d,
    0xfac1287c93bbf3a2,
    0x90f82b552ceb4228,
    0x90b4ab38d2525f0c,
    0xc2d64cf87e7d2315,
    0xa19ac9598af08005,
    0xe1e6b41d0922e27a,
    0x655705c7c59174e2,
    0x9069ee5b9429b677,
    0xc1ed9fa40654c770,
    0x20c5b7125571630d,
    0xb7c391f853d76cc7,
    0x1ac5899e7b6f95ab,
    0x6d9ecbb55392b170,
    0xde0ba771a93d5864,
    0x053f15838ec330c5,
];
const SIDE_HASH: u64 = 0xe48fe3e0fb244264;

#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Debug, Hash, Default, Serialize, Deserialize)]
pub struct ZHash(pub u64);

impl ZHash {
    pub fn toggle_piece_at_pos(&mut self, piece: ChessPiece, color: PieceColor, pos: usize) {
        let hash_pos = piece as usize * color as usize + pos;
        self.0 ^= &ZHASH_TABLE[hash_pos];
    }

    pub fn toggle_enpassant(&mut self, pos: usize) {
        self.0 ^= &EN_PASSANT_TABLE[pos];
    }

    pub fn swap_castling_rights(&mut self, old: &CastlingRights, new: &CastlingRights) {
        self.0 ^= &CASTLING_HASHES[old.index()];
        self.0 ^= &CASTLING_HASHES[new.index()];
    }

    pub fn toggle_side(&mut self) {
        self.0 ^= SIDE_HASH;
    }
}
