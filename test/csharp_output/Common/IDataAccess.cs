using System.IO;

namespace Cfg.Common;

public interface IDataAccess {
    string GetString(IDataPath path);
    IDataPath JoinPath(IDataPath path, string item);
}

public interface IDataPath {

}