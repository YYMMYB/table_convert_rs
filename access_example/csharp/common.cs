using System;
using System.Collections.Generic;
using System.IO;
using System.Text;

// 你的Common命名空间
using __Gen.Cfg.Common;

public class DataAccess : IDataAccess {
    public string GetData(IDataPath path) {
        var p = (DataPath)path;
        return File.ReadAllText(p.path);
    }

    public IDataPath JoinPath(IDataPath path, string item) {
        return (path as DataPath).Join(item);
    }

    public IDataPath RootPath() {
        // 你自己的json路径
        return new DataPath("D:\\Project\\cs_test\\cs_test\\Gen\\json\\");
    }
}

public record class DataPath : IDataPath {
    public string path;

    public DataPath(string p) { path = p; }

    public DataPath Join(string s) {
        var npath = Path.Join(path, s);
        return new DataPath(npath);
    }
}