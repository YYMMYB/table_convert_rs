using System;
using System.Collections.Generic;
using System.Text.Json.Serialization;
using Cfg.Common;

namespace Cfg.Types.命名空间;
public class Mod {
    public Dictionary<int, Cfg.Types.命名空间.a.a_item> a;
    public Dictionary<int, Cfg.Types.命名空间.a2.a2_item> a2;


    public static Cfg.Types.命名空间.Mod Load(IDataAccess access, IDataPath folder) {
        var tables = new Cfg.Types.命名空间.Mod();

        // 数据表
        {
        var s = access.GetString(access.JoinPath(folder, "a.json"));
        tables.a = JsonSerializer.Deserialize<Dictionary<int, Cfg.Types.命名空间.a.a_item>>(s, Cfg.Common.Utils.Options);
        }
        {
        var s = access.GetString(access.JoinPath(folder, "a2.json"));
        tables.a2 = JsonSerializer.Deserialize<Dictionary<int, Cfg.Types.命名空间.a2.a2_item>>(s, Cfg.Common.Utils.Options);
        }

        // 子模块
        return tables;
    }
}
