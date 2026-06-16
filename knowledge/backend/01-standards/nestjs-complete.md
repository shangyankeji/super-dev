---
id: nestjs-complete
title: NestJS 完整指南
domain: backend
category: 01-standards
difficulty: intermediate
tags: [backend, complete, nestjs, websocket, 依赖注入系统, 微服务, 快速开始, 数据库集成]
quality_score: 70
last_updated: 2026-06-15
---
# NestJS 完整指南

## 概述

NestJS 是一个用于构建高效、可扩展 Node.js 服务端应用的渐进式框架。它完全使用 TypeScript 构建（同时兼容纯 JavaScript），融合了面向对象编程 (OOP)、函数式编程 (FP) 和函数式响应式编程 (FRP) 的理念。底层默认使用 Express，但也支持切换到 Fastify 以获得更高性能。

### NestJS vs Express vs Fastify

| 特性 | NestJS | Express | Fastify |
|------|--------|---------|---------|
| 架构模式 | 模块化 + 依赖注入 | 无约定，极简 | 插件体系 |
| TypeScript 支持 | 原生一等公民 | 需手动配置 | 内置支持 |
| 性能（req/s 基准） | 与底层适配器一致 | ~15k | ~30k |
| 学习曲线 | 中高（Angular 风格） | 低 | 低-中 |
| 依赖注入 | 内置 IoC 容器 | 无 | 无 |
| 微服务支持 | 内置多种 Transport | 无（需第三方） | 无（需第三方） |
| WebSocket | 内置 Gateway | 需 socket.io | 需插件 |
| GraphQL | 官方模块 | 需 Apollo 手动集成 | 需 Mercurius |
| 测试工具 | Testing 模块 + 自动 Mock | 需自行搭建 | 需自行搭建 |
| 企业级就绪度 | 高 | 中 | 中 |

### 何时选择 NestJS

✅ **大型团队协作**: 强约定 + 模块化架构减少代码风格分歧
✅ **微服务体系**: 内置 Transport 层，支持 Redis/Kafka/gRPC/NATS/MQTT
✅ **企业级后端**: 完善的认证/授权/缓存/队列/日志/健康检查生态
✅ **全栈 TypeScript**: 前后端共享类型定义，配合 Monorepo 高效开发
✅ **需要长期维护的系统**: 模块边界清晰，重构成本低

❌ **简单 API / 原型验证**: 过度工程化，Express/Fastify 更轻便
❌ **极致性能场景**: 框架层带来一定开销，考虑 Fastify 或 Go

---

## 快速开始

### 安装

```bash
# 使用 CLI 创建项目（推荐）
npm i -g @nestjs/cli
nest new my-project

# 手动安装核心包
npm i @nestjs/core @nestjs/common @nestjs/platform-express reflect-metadata rxjs
```

### Hello World

```typescript
// main.ts
import { NestFactory } from '@nestjs/core';
import { AppModule } from './app.module';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  await app.listen(3000);
}
bootstrap();

// app.module.ts
import { Module } from '@nestjs/common';
import { AppController } from './app.controller';
import { AppService } from './app.service';

@Module({
  imports: [],
  controllers: [AppController],
  providers: [AppService],
})
export class AppModule {}

// app.controller.ts
import { Controller, Get } from '@nestjs/common';
import { AppService } from './app.service';

@Controller()
export class AppController {
  constructor(private readonly appService: AppService) {}

  @Get()
  getHello(): string {
    return this.appService.getHello();
  }
}

// app.service.ts
import { Injectable } from '@nestjs/common';

@Injectable()
export class AppService {
  getHello(): string {
    return 'Hello World!';
  }
}
```

### CLI 常用命令

```bash
nest generate module users        # 生成模块
nest generate controller users    # 生成控制器
nest generate service users       # 生成服务
nest generate resource users      # 一键生成完整 CRUD 资源（模块+控制器+服务+DTO+实体）
nest build                        # 编译项目
nest start --watch                # 开发模式热重载
```

---

## 核心概念

### 1. 模块 (Modules)

模块是 NestJS 应用的基本组织单元。每个应用至少有一个根模块 (AppModule)。

```typescript
import { Module, Global } from '@nestjs/common';
import { UsersController } from './users.controller';
import { UsersService } from './users.service';
import { AuthModule } from '../auth/auth.module';

@Module({
  imports: [AuthModule],        // 导入其他模块
  controllers: [UsersController], // 注册控制器
  providers: [UsersService],      // 注册提供者
  exports: [UsersService],        // 导出供其他模块使用
})
export class UsersModule {}

// 全局模块 — 导入一次，全局可用
@Global()
@Module({
  providers: [ConfigService],
  exports: [ConfigService],
})
export class ConfigModule {}

// 动态模块 — 根据参数动态配置
@Module({})
export class DatabaseModule {
  static forRoot(options: DatabaseOptions): DynamicModule {
    return {
      module: DatabaseModule,
      providers: [
        {
          provide: 'DATABASE_OPTIONS',
          useValue: options,
        },
        DatabaseService,
      ],
      exports: [DatabaseService],
      global: true,
    };
  }

  static forFeature(entities: Type[]): DynamicModule {
    const providers = entities.map((entity) => ({
      provide: getRepositoryToken(entity),
      useFactory: (ds: DataSource) => ds.getRepository(entity),
      inject: [DataSource],
    }));
    return {
      module: DatabaseModule,
      providers,
      exports: providers,
    };
  }
}
```

### 2. 控制器 (Controllers)

控制器负责处理传入请求并返回响应。

```typescript
import {
  Controller, Get, Post, Put, Delete, Patch,
  Param, Query, Body, Headers, Ip,
  HttpCode, HttpStatus, Redirect, Header,
  Res, Req, ParseIntPipe, ParseUUIDPipe,
  UseGuards, UseInterceptors, UsePipes,
} from '@nestjs/common';
import { Request, Response } from 'express';

@Controller('users') // 路由前缀 /users
export class UsersController {
  constructor(private readonly usersService: UsersService) {}

  @Get()
  findAll(@Query('page', new DefaultValuePipe(1), ParseIntPipe) page: number) {
    return this.usersService.findAll(page);
  }

  @Get(':id')
  findOne(@Param('id', ParseUUIDPipe) id: string) {
    return this.usersService.findOne(id);
  }

  @Post()
  @HttpCode(HttpStatus.CREATED)
  create(@Body() createUserDto: CreateUserDto) {
    return this.usersService.create(createUserDto);
  }

  @Put(':id')
  update(@Param('id') id: string, @Body() updateUserDto: UpdateUserDto) {
    return this.usersService.update(id, updateUserDto);
  }

  @Delete(':id')
  @HttpCode(HttpStatus.NO_CONTENT)
  remove(@Param('id') id: string) {
    return this.usersService.remove(id);
  }

  // 子路由分组
  @Get(':id/orders')
  findUserOrders(@Param('id') id: string) {
    return this.usersService.findOrders(id);
  }

  // 自定义响应（绕过序列化）
  @Get(':id/avatar')
  async getAvatar(@Param('id') id: string, @Res() res: Response) {
    const stream = await this.usersService.getAvatar(id);
    stream.pipe(res);
  }
}

// 版本控制
@Controller({ path: 'users', version: '2' })
export class UsersV2Controller {
  // /v2/users
}
```

### 3. 提供者 (Providers)

提供者是 NestJS 依赖注入的核心。任何被 `@Injectable()` 修饰的类都可作为提供者。

```typescript
import { Injectable, Scope, Inject } from '@nestjs/common';

@Injectable()
export class UsersService {
  constructor(
    @Inject('USER_REPOSITORY') private userRepo: Repository<User>,
    private readonly configService: ConfigService,
  ) {}

  async findAll(page: number): Promise<PaginatedResult<User>> {
    const take = this.configService.get<number>('PAGE_SIZE', 20);
    const skip = (page - 1) * take;
    const [data, total] = await this.userRepo.findAndCount({ take, skip });
    return { data, total, page, pageSize: take };
  }

  async findOne(id: string): Promise<User> {
    const user = await this.userRepo.findOne({ where: { id } });
    if (!user) {
      throw new NotFoundException(`User #${id} not found`);
    }
    return user;
  }

  async create(dto: CreateUserDto): Promise<User> {
    const user = this.userRepo.create(dto);
    return this.userRepo.save(user);
  }
}
```

### 4. 中间件 (Middleware)

在路由处理器之前执行，可访问请求/响应对象。

```typescript
import { Injectable, NestMiddleware } from '@nestjs/common';
import { Request, Response, NextFunction } from 'express';

@Injectable()
export class LoggerMiddleware implements NestMiddleware {
  use(req: Request, res: Response, next: NextFunction) {
    const start = Date.now();
    res.on('finish', () => {
      const duration = Date.now() - start;
      console.log(`${req.method} ${req.url} ${res.statusCode} - ${duration}ms`);
    });
    next();
  }
}

// 在模块中注册
@Module({})
export class AppModule implements NestModule {
  configure(consumer: MiddlewareConsumer) {
    consumer
      .apply(LoggerMiddleware, CorsMiddleware)
      .exclude({ path: 'health', method: RequestMethod.GET })
      .forRoutes('*');
  }
}

// 函数式中间件（简单场景）
export function helmet(req: Request, res: Response, next: NextFunction) {
  // 安全头设置
  next();
}
```

### 5. 管道 (Pipes)

用于数据转换和验证。在路由处理器执行之前处理参数。

```typescript
import {
  PipeTransform, Injectable, ArgumentMetadata,
  BadRequestException,
} from '@nestjs/common';
import { validate } from 'class-validator';
import { plainToInstance } from 'class-transformer';

// 全局验证管道（推荐在 main.ts 设置）
async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  app.useGlobalPipes(new ValidationPipe({
    whitelist: true,           // 剥离非 DTO 定义的属性
    forbidNonWhitelisted: true, // 存在非白名单属性时抛出异常
    transform: true,            // 自动类型转换
    transformOptions: {
      enableImplicitConversion: true,
    },
  }));
  await app.listen(3000);
}

// 自定义管道
@Injectable()
export class ParseDatePipe implements PipeTransform<string, Date> {
  transform(value: string, metadata: ArgumentMetadata): Date {
    const date = new Date(value);
    if (isNaN(date.getTime())) {
      throw new BadRequestException(`"${value}" is not a valid date`);
    }
    return date;
  }
}

// DTO 验证示例
import { IsString, IsEmail, MinLength, IsOptional, IsEnum } from 'class-validator';

export class CreateUserDto {
  @IsString()
  @MinLength(2)
  name: string;

  @IsEmail()
  email: string;

  @IsString()
  @MinLength(8)
  password: string;

  @IsOptional()
  @IsEnum(UserRole)
  role?: UserRole;
}
```

### 6. 守卫 (Guards)

决定请求是否被路由处理器处理。用于认证和授权。

```typescript
import {
  Injectable, CanActivate, ExecutionContext,
  SetMetadata, UseGuards, applyDecorators,
} from '@nestjs/common';
import { Reflector } from '@nestjs/core';

// 角色守卫
@Injectable()
export class RolesGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  canActivate(context: ExecutionContext): boolean {
    const requiredRoles = this.reflector.getAllAndOverride<Role[]>('roles', [
      context.getHandler(),
      context.getClass(),
    ]);
    if (!requiredRoles) return true;

    const { user } = context.switchToHttp().getRequest();
    return requiredRoles.some((role) => user.roles?.includes(role));
  }
}

// 角色装饰器
export const Roles = (...roles: Role[]) => SetMetadata('roles', roles);

// 组合装饰器（推荐模式）
export function Auth(...roles: Role[]) {
  return applyDecorators(
    Roles(...roles),
    UseGuards(JwtAuthGuard, RolesGuard),
  );
}

// 使用
@Controller('admin')
export class AdminController {
  @Get('dashboard')
  @Auth(Role.ADMIN)
  getDashboard() {
    return { message: 'Admin dashboard' };
  }
}
```

### 7. 拦截器 (Interceptors)

在路由处理器之前/之后执行额外逻辑。可用于日志、缓存、响应映射、超时等。

```typescript
import {
  Injectable, NestInterceptor, ExecutionContext, CallHandler,
} from '@nestjs/common';
import { Observable, map, tap, timeout, catchError } from 'rxjs';

// 响应包装拦截器
@Injectable()
export class TransformInterceptor<T> implements NestInterceptor<T, Response<T>> {
  intercept(context: ExecutionContext, next: CallHandler): Observable<Response<T>> {
    return next.handle().pipe(
      map((data) => ({
        statusCode: context.switchToHttp().getResponse().statusCode,
        data,
        timestamp: new Date().toISOString(),
      })),
    );
  }
}

// 日志拦截器
@Injectable()
export class LoggingInterceptor implements NestInterceptor {
  private readonly logger = new Logger(LoggingInterceptor.name);

  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    const req = context.switchToHttp().getRequest();
    const { method, url } = req;
    const now = Date.now();

    return next.handle().pipe(
      tap(() => this.logger.log(`${method} ${url} - ${Date.now() - now}ms`)),
    );
  }
}

// 超时拦截器
@Injectable()
export class TimeoutInterceptor implements NestInterceptor {
  intercept(context: ExecutionContext, next: CallHandler): Observable<any> {
    return next.handle().pipe(
      timeout(5000),
      catchError((err) => {
        if (err.name === 'TimeoutError') {
          throw new RequestTimeoutException();
        }
        throw err;
      }),
    );
  }
}
```

### 8. 异常过滤器 (Exception Filters)

捕获并处理未处理的异常，统一错误响应格式。

```typescript
import {
  ExceptionFilter, Catch, ArgumentsHost,
  HttpException, HttpStatus, Logger,
} from '@nestjs/common';
import { Request, Response } from 'express';

@Catch()
export class AllExceptionsFilter implements ExceptionFilter {
  private readonly logger = new Logger(AllExceptionsFilter.name);

  catch(exception: unknown, host: ArgumentsHost) {
    const ctx = host.switchToHttp();
    const response = ctx.getResponse<Response>();
    const request = ctx.getRequest<Request>();

    const status =
      exception instanceof HttpException
        ? exception.getStatus()
        : HttpStatus.INTERNAL_SERVER_ERROR;

    const message =
      exception instanceof HttpException
        ? exception.getResponse()
        : 'Internal server error';

    const errorResponse = {
      statusCode: status,
      timestamp: new Date().toISOString(),
      path: request.url,
      method: request.method,
      message: typeof message === 'string' ? message : (message as any).message,
    };

    this.logger.error(
      `${request.method} ${request.url} ${status}`,
      exception instanceof Error ? exception.stack : '',
    );

    response.status(status).json(errorResponse);
  }
}

// 自定义业务异常
export class BusinessException extends HttpException {
  constructor(code: string, message: string, status = HttpStatus.BAD_REQUEST) {
    super({ code, message }, status);
  }
}
```

---

## 依赖注入系统

### 提供者作用域

```typescript
import { Injectable, Scope } from '@nestjs/common';

// 默认: 单例（推荐，整个应用生命周期只创建一个实例）
@Injectable()
export class SingletonService {}

// 请求作用域（每个请求创建新实例，注意性能影响）
@Injectable({ scope: Scope.REQUEST })
export class RequestScopedService {
  constructor(@Inject(REQUEST) private request: Request) {}
}

// 瞬态作用域（每次注入创建新实例）
@Injectable({ scope: Scope.TRANSIENT })
export class TransientService {}
```

> **性能警告**: 请求作用域会导致依赖链上的所有提供者都变为请求作用域，显著增加实例化开销。除非确实需要访问请求对象，否则使用默认单例作用域。

### 自定义提供者

```typescript
@Module({
  providers: [
    // useClass — 标准类提供者
    { provide: UsersService, useClass: UsersService },

    // useValue — 值提供者（常量、配置、Mock）
    { provide: 'API_KEY', useValue: process.env.API_KEY },
    { provide: 'CONFIG', useValue: { retries: 3, timeout: 5000 } },

    // useFactory — 工厂提供者（异步初始化、条件逻辑）
    {
      provide: 'ASYNC_CONNECTION',
      useFactory: async (configService: ConfigService) => {
        const conn = await createConnection(configService.get('DATABASE_URL'));
        return conn;
      },
      inject: [ConfigService],
    },

    // useExisting — 别名提供者
    { provide: 'AliasedService', useExisting: UsersService },
  ],
})
export class AppModule {}
```

### 循环依赖解决

```typescript
// 方法 1: forwardRef（推荐）
@Injectable()
export class CatsService {
  constructor(
    @Inject(forwardRef(() => DogsService))
    private dogsService: DogsService,
  ) {}
}

@Injectable()
export class DogsService {
  constructor(
    @Inject(forwardRef(() => CatsService))
    private catsService: CatsService,
  ) {}
}

// 模块级别同理
@Module({
  imports: [forwardRef(() => DogsModule)],
})
export class CatsModule {}

// 方法 2: ModuleRef（运行时解析，避免循环）
@Injectable()
export class CatsService implements OnModuleInit {
  private dogsService: DogsService;

  constructor(private moduleRef: ModuleRef) {}

  onModuleInit() {
    this.dogsService = this.moduleRef.get(DogsService, { strict: false });
  }
}
```

> **最佳实践**: 循环依赖通常是架构设计的坏味道。优先考虑提取公共逻辑到第三个服务，或使用事件驱动模式解耦。

---

## 数据库集成

### TypeORM 集成

```bash
npm i @nestjs/typeorm typeorm pg
```

```typescript
// app.module.ts
@Module({
  imports: [
    TypeOrmModule.forRootAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (config: ConfigService) => ({
        type: 'postgres',
        host: config.get('DB_HOST'),
        port: config.get<number>('DB_PORT', 5432),
        username: config.get('DB_USER'),
        password: config.get('DB_PASS'),
        database: config.get('DB_NAME'),
        entities: [__dirname + '/**/*.entity{.ts,.js}'],
        synchronize: false, // 生产环境必须为 false
        logging: config.get('NODE_ENV') === 'development',
        migrations: [__dirname + '/migrations/*{.ts,.js}'],
      }),
    }),
  ],
})
export class AppModule {}

// user.entity.ts
@Entity('users')
export class User {
  @PrimaryGeneratedColumn('uuid')
  id: string;

  @Column({ unique: true })
  email: string;

  @Column()
  name: string;

  @Column({ select: false })
  password: string;

  @OneToMany(() => Order, (order) => order.user)
  orders: Order[];

  @CreateDateColumn()
  createdAt: Date;

  @UpdateDateColumn()
  updatedAt: Date;
}

// users.module.ts
@Module({
  imports: [TypeOrmModule.forFeature([User])],
  providers: [UsersService],
  controllers: [UsersController],
})
export class UsersModule {}

// users.service.ts (Repository 模式)
@Injectable()
export class UsersService {
  constructor(
    @InjectRepository(User)
    private readonly userRepo: Repository<User>,
  ) {}

  async findByEmail(email: string): Promise<User | null> {
    return this.userRepo.findOne({
      where: { email },
      relations: ['orders'],
    });
  }
}
```

### TypeORM 迁移

```bash
# 生成迁移
npx typeorm migration:generate -d src/data-source.ts src/migrations/AddUserTable

# 运行迁移
npx typeorm migration:run -d src/data-source.ts

# 回滚
npx typeorm migration:revert -d src/data-source.ts
```

### TypeORM 事务

```typescript
@Injectable()
export class OrdersService {
  constructor(private readonly dataSource: DataSource) {}

  async createOrder(userId: string, items: OrderItemDto[]): Promise<Order> {
    const queryRunner = this.dataSource.createQueryRunner();
    await queryRunner.connect();
    await queryRunner.startTransaction();

    try {
      const order = queryRunner.manager.create(Order, { userId });
      const savedOrder = await queryRunner.manager.save(order);

      for (const item of items) {
        // 扣减库存
        const product = await queryRunner.manager.findOne(Product, {
          where: { id: item.productId },
          lock: { mode: 'pessimistic_write' },
        });
        if (!product || product.stock < item.quantity) {
          throw new BadRequestException(`Insufficient stock for product ${item.productId}`);
        }
        product.stock -= item.quantity;
        await queryRunner.manager.save(product);

        // 创建订单项
        const orderItem = queryRunner.manager.create(OrderItem, {
          orderId: savedOrder.id, ...item,
        });
        await queryRunner.manager.save(orderItem);
      }

      await queryRunner.commitTransaction();
      return savedOrder;
    } catch (err) {
      await queryRunner.rollbackTransaction();
      throw err;
    } finally {
      await queryRunner.release();
    }
  }
}
```

### Prisma 集成

```bash
npm i prisma @prisma/client
npx prisma init
```

```typescript
// prisma.service.ts
@Injectable()
export class PrismaService extends PrismaClient implements OnModuleInit, OnModuleDestroy {
  async onModuleInit() {
    await this.$connect();
  }

  async onModuleDestroy() {
    await this.$disconnect();
  }

  // 清理用于测试
  async cleanDatabase() {
    if (process.env.NODE_ENV !== 'test') {
      throw new Error('cleanDatabase is only for test environment');
    }
    const models = Reflect.ownKeys(this).filter((key) => typeof key === 'string' && !key.startsWith('_'));
    return Promise.all(
      models.map((model) => (this as any)[model]?.deleteMany?.()),
    );
  }
}

// prisma.module.ts
@Global()
@Module({
  providers: [PrismaService],
  exports: [PrismaService],
})
export class PrismaModule {}

// users.service.ts (Prisma)
@Injectable()
export class UsersService {
  constructor(private readonly prisma: PrismaService) {}

  async findAll(params: { skip?: number; take?: number; where?: Prisma.UserWhereInput }) {
    const { skip, take, where } = params;
    return this.prisma.user.findMany({ skip, take, where, include: { orders: true } });
  }

  // Prisma 事务
  async transferCredits(fromId: string, toId: string, amount: number) {
    return this.prisma.$transaction(async (tx) => {
      const sender = await tx.user.update({
        where: { id: fromId },
        data: { credits: { decrement: amount } },
      });
      if (sender.credits < 0) {
        throw new BadRequestException('Insufficient credits');
      }
      await tx.user.update({
        where: { id: toId },
        data: { credits: { increment: amount } },
      });
    });
  }
}
```

---

## 认证与授权

### Passport + JWT 认证

```bash
npm i @nestjs/passport passport passport-local passport-jwt @nestjs/jwt
npm i -D @types/passport-local @types/passport-jwt
```

```typescript
// auth.module.ts
@Module({
  imports: [
    UsersModule,
    PassportModule.register({ defaultStrategy: 'jwt' }),
    JwtModule.registerAsync({
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (config: ConfigService) => ({
        secret: config.get('JWT_SECRET'),
        signOptions: { expiresIn: '15m' },
      }),
    }),
  ],
  providers: [AuthService, LocalStrategy, JwtStrategy],
  controllers: [AuthController],
  exports: [AuthService],
})
export class AuthModule {}

// jwt.strategy.ts
@Injectable()
export class JwtStrategy extends PassportStrategy(Strategy) {
  constructor(
    private configService: ConfigService,
    private usersService: UsersService,
  ) {
    super({
      jwtFromRequest: ExtractJwt.fromAuthHeaderAsBearerToken(),
      ignoreExpiration: false,
      secretOrKey: configService.get('JWT_SECRET'),
    });
  }

  async validate(payload: JwtPayload): Promise<User> {
    const user = await this.usersService.findOne(payload.sub);
    if (!user) throw new UnauthorizedException();
    return user;
  }
}

// auth.service.ts
@Injectable()
export class AuthService {
  constructor(
    private usersService: UsersService,
    private jwtService: JwtService,
  ) {}

  async validateUser(email: string, password: string): Promise<User | null> {
    const user = await this.usersService.findByEmail(email);
    if (user && (await bcrypt.compare(password, user.password))) {
      return user;
    }
    return null;
  }

  async login(user: User) {
    const payload: JwtPayload = { sub: user.id, email: user.email, roles: user.roles };
    return {
      accessToken: this.jwtService.sign(payload),
      refreshToken: this.jwtService.sign(payload, { expiresIn: '7d' }),
    };
  }

  async refreshToken(token: string) {
    try {
      const payload = this.jwtService.verify(token);
      const user = await this.usersService.findOne(payload.sub);
      if (!user) throw new UnauthorizedException();
      return this.login(user);
    } catch {
      throw new UnauthorizedException('Invalid refresh token');
    }
  }
}
```

### RBAC（基于角色的访问控制）

```typescript
export enum Role {
  USER = 'user',
  ADMIN = 'admin',
  MODERATOR = 'moderator',
}

// roles.decorator.ts
export const ROLES_KEY = 'roles';
export const Roles = (...roles: Role[]) => SetMetadata(ROLES_KEY, roles);

// roles.guard.ts
@Injectable()
export class RolesGuard implements CanActivate {
  constructor(private reflector: Reflector) {}

  canActivate(context: ExecutionContext): boolean {
    const requiredRoles = this.reflector.getAllAndOverride<Role[]>(ROLES_KEY, [
      context.getHandler(),
      context.getClass(),
    ]);
    if (!requiredRoles) return true;
    const { user } = context.switchToHttp().getRequest();
    return requiredRoles.some((role) => user.roles?.includes(role));
  }
}

// 使用
@Controller('posts')
@UseGuards(JwtAuthGuard, RolesGuard)
export class PostsController {
  @Delete(':id')
  @Roles(Role.ADMIN, Role.MODERATOR)
  remove(@Param('id') id: string) {
    return this.postsService.remove(id);
  }
}
```

### CASL 权限管理

```bash
npm i @casl/ability
```

```typescript
// casl-ability.factory.ts
@Injectable()
export class CaslAbilityFactory {
  createForUser(user: User) {
    const { can, cannot, build } = new AbilityBuilder<Ability>(Ability);

    if (user.roles.includes(Role.ADMIN)) {
      can('manage', 'all');
    } else {
      can('read', 'Article');
      can('create', 'Article');
      can('update', 'Article', { authorId: user.id }); // 只能编辑自己的文章
      cannot('delete', 'Article');
    }

    return build();
  }
}

// policies.guard.ts
@Injectable()
export class PoliciesGuard implements CanActivate {
  constructor(
    private reflector: Reflector,
    private caslAbilityFactory: CaslAbilityFactory,
  ) {}

  canActivate(context: ExecutionContext): boolean {
    const policyHandlers = this.reflector.get<PolicyHandler[]>('check_policies', context.getHandler()) || [];
    const { user } = context.switchToHttp().getRequest();
    const ability = this.caslAbilityFactory.createForUser(user);
    return policyHandlers.every((handler) => handler(ability));
  }
}
```

---

## WebSocket

### Gateway 基础

```bash
npm i @nestjs/websockets @nestjs/platform-socket.io socket.io
```

```typescript
import {
  WebSocketGateway, WebSocketServer,
  SubscribeMessage, MessageBody,
  ConnectedSocket, OnGatewayInit,
  OnGatewayConnection, OnGatewayDisconnect,
  WsException,
} from '@nestjs/websockets';
import { Server, Socket } from 'socket.io';

@WebSocketGateway({
  cors: { origin: '*' },
  namespace: '/chat',
})
export class ChatGateway
  implements OnGatewayInit, OnGatewayConnection, OnGatewayDisconnect
{
  @WebSocketServer()
  server: Server;

  private readonly logger = new Logger(ChatGateway.name);

  afterInit(server: Server) {
    this.logger.log('WebSocket Gateway initialized');
  }

  handleConnection(client: Socket) {
    this.logger.log(`Client connected: ${client.id}`);
  }

  handleDisconnect(client: Socket) {
    this.logger.log(`Client disconnected: ${client.id}`);
  }

  @SubscribeMessage('sendMessage')
  handleMessage(
    @MessageBody() data: { room: string; message: string },
    @ConnectedSocket() client: Socket,
  ) {
    // 向房间广播（不包括发送者）
    client.to(data.room).emit('newMessage', {
      sender: client.id,
      message: data.message,
      timestamp: new Date(),
    });
    return { event: 'messageSent', data: { success: true } };
  }

  @SubscribeMessage('joinRoom')
  handleJoinRoom(
    @MessageBody() room: string,
    @ConnectedSocket() client: Socket,
  ) {
    client.join(room);
    this.server.to(room).emit('userJoined', { userId: client.id });
  }

  @SubscribeMessage('leaveRoom')
  handleLeaveRoom(
    @MessageBody() room: string,
    @ConnectedSocket() client: Socket,
  ) {
    client.leave(room);
    this.server.to(room).emit('userLeft', { userId: client.id });
  }
}
```

### WebSocket 认证

```typescript
@WebSocketGateway()
export class AuthenticatedGateway {
  @WebSocketServer() server: Server;

  afterInit(server: Server) {
    server.use(async (socket: Socket, next) => {
      try {
        const token = socket.handshake.auth.token
          || socket.handshake.headers.authorization?.split(' ')[1];
        if (!token) throw new WsException('Missing token');
        const payload = this.jwtService.verify(token);
        socket.data.user = payload;
        next();
      } catch (err) {
        next(new Error('Unauthorized'));
      }
    });
  }
}
```

---

## 微服务

### Transport 层概览

NestJS 微服务抽象了通信层，支持多种传输协议：

```typescript
// TCP（默认）
const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
  transport: Transport.TCP,
  options: { host: '0.0.0.0', port: 3001 },
});

// Redis
const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
  transport: Transport.REDIS,
  options: { host: 'localhost', port: 6379 },
});

// Kafka
const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
  transport: Transport.KAFKA,
  options: {
    client: { brokers: ['localhost:9092'], clientId: 'orders-service' },
    consumer: { groupId: 'orders-consumer' },
  },
});

// gRPC
const app = await NestFactory.createMicroservice<MicroserviceOptions>(AppModule, {
  transport: Transport.GRPC,
  options: {
    package: 'orders',
    protoPath: join(__dirname, 'orders.proto'),
    url: '0.0.0.0:50051',
  },
});
```

### 消息模式与事件

```typescript
// 微服务端 — 处理消息
@Controller()
export class OrdersController {
  // 请求-响应模式（同步通信）
  @MessagePattern({ cmd: 'get_order' })
  getOrder(@Payload() data: { orderId: string }) {
    return this.ordersService.findOne(data.orderId);
  }

  // 事件模式（异步通信，无回复）
  @EventPattern('order_created')
  handleOrderCreated(@Payload() data: OrderCreatedEvent) {
    this.notificationService.sendOrderConfirmation(data);
  }
}

// 客户端 — 发送消息
@Module({
  imports: [
    ClientsModule.register([{
      name: 'ORDERS_SERVICE',
      transport: Transport.REDIS,
      options: { host: 'localhost', port: 6379 },
    }]),
  ],
})
export class ApiGatewayModule {}

@Injectable()
export class ApiGatewayService {
  constructor(@Inject('ORDERS_SERVICE') private ordersClient: ClientProxy) {}

  getOrder(orderId: string): Observable<Order> {
    return this.ordersClient.send({ cmd: 'get_order' }, { orderId });
  }

  createOrder(dto: CreateOrderDto) {
    this.ordersClient.emit('order_created', dto); // 单向事件
  }
}
```

### 混合应用（HTTP + 微服务）

```typescript
async function bootstrap() {
  const app = await NestFactory.create(AppModule);

  // 同时连接 Redis 和 Kafka 微服务
  app.connectMicroservice<MicroserviceOptions>({
    transport: Transport.REDIS,
    options: { host: 'localhost', port: 6379 },
  });
  app.connectMicroservice<MicroserviceOptions>({
    transport: Transport.KAFKA,
    options: {
      client: { brokers: ['localhost:9092'] },
      consumer: { groupId: 'hybrid-consumer' },
    },
  });

  await app.startAllMicroservices();
  await app.listen(3000);
}
```

---

## GraphQL

### Code-First 方式

```bash
npm i @nestjs/graphql @nestjs/apollo @apollo/server graphql
```

```typescript
// app.module.ts
@Module({
  imports: [
    GraphQLModule.forRoot<ApolloDriverConfig>({
      driver: ApolloDriver,
      autoSchemaFile: join(process.cwd(), 'src/schema.gql'),
      sortSchema: true,
      playground: process.env.NODE_ENV !== 'production',
      context: ({ req }) => ({ req }),
    }),
  ],
})
export class AppModule {}

// user.model.ts
@ObjectType()
export class UserModel {
  @Field(() => ID)
  id: string;

  @Field()
  email: string;

  @Field()
  name: string;

  @Field(() => [OrderModel], { nullable: true })
  orders?: OrderModel[];

  @Field()
  createdAt: Date;
}

// users.resolver.ts
@Resolver(() => UserModel)
export class UsersResolver {
  constructor(private readonly usersService: UsersService) {}

  @Query(() => [UserModel], { name: 'users' })
  findAll(
    @Args('page', { type: () => Int, defaultValue: 1 }) page: number,
  ) {
    return this.usersService.findAll(page);
  }

  @Query(() => UserModel, { name: 'user' })
  findOne(@Args('id', { type: () => ID }) id: string) {
    return this.usersService.findOne(id);
  }

  @Mutation(() => UserModel)
  createUser(@Args('input') input: CreateUserInput) {
    return this.usersService.create(input);
  }

  @ResolveField(() => [OrderModel])
  orders(@Parent() user: UserModel) {
    return this.ordersService.findByUserId(user.id);
  }
}
```

### DataLoader（N+1 问题解决）

```typescript
import DataLoader from 'dataloader';

@Injectable({ scope: Scope.REQUEST })
export class OrdersLoader {
  constructor(private readonly ordersService: OrdersService) {}

  readonly batchByUserId = new DataLoader<string, Order[]>(async (userIds: string[]) => {
    const orders = await this.ordersService.findByUserIds([...userIds]);
    const ordersMap = new Map<string, Order[]>();
    orders.forEach((order) => {
      const existing = ordersMap.get(order.userId) || [];
      existing.push(order);
      ordersMap.set(order.userId, existing);
    });
    return userIds.map((id) => ordersMap.get(id) || []);
  });
}

// resolver 中使用
@ResolveField(() => [OrderModel])
orders(@Parent() user: UserModel) {
  return this.ordersLoader.batchByUserId.load(user.id);
}
```

### 订阅 (Subscriptions)

```typescript
// app.module.ts 启用订阅
GraphQLModule.forRoot<ApolloDriverConfig>({
  driver: ApolloDriver,
  autoSchemaFile: true,
  subscriptions: {
    'graphql-ws': true,         // 推荐：graphql-ws 协议
    'subscriptions-transport-ws': false,
  },
});

// resolver
@Resolver()
export class MessagesResolver {
  @Subscription(() => MessageModel, {
    filter: (payload, variables) =>
      payload.messageAdded.channelId === variables.channelId,
  })
  messageAdded(@Args('channelId') channelId: string) {
    return pubSub.asyncIterator('messageAdded');
  }

  @Mutation(() => MessageModel)
  async sendMessage(@Args('input') input: SendMessageInput) {
    const message = await this.messagesService.create(input);
    pubSub.publish('messageAdded', { messageAdded: message });
    return message;
  }
}
```

---

## 队列处理

### Bull / BullMQ 集成

```bash
npm i @nestjs/bullmq bullmq
```

```typescript
// app.module.ts
@Module({
  imports: [
    BullModule.forRoot({
      connection: { host: 'localhost', port: 6379 },
    }),
    BullModule.registerQueue(
      { name: 'email' },
      { name: 'report' },
    ),
  ],
})
export class AppModule {}

// email.producer.ts
@Injectable()
export class EmailService {
  constructor(@InjectQueue('email') private emailQueue: Queue) {}

  async sendWelcomeEmail(userId: string) {
    await this.emailQueue.add('welcome', { userId }, {
      delay: 5000,             // 延迟 5 秒发送
      attempts: 3,             // 失败重试 3 次
      backoff: { type: 'exponential', delay: 2000 },
      removeOnComplete: 100,   // 保留最近 100 个完成任务
      removeOnFail: 500,       // 保留最近 500 个失败任务
    });
  }

  async sendBulkEmails(userIds: string[]) {
    const jobs = userIds.map((userId) => ({
      name: 'bulk',
      data: { userId },
      opts: { priority: 10 },
    }));
    await this.emailQueue.addBulk(jobs);
  }
}

// email.consumer.ts
@Processor('email')
export class EmailConsumer extends WorkerHost {
  private readonly logger = new Logger(EmailConsumer.name);

  async process(job: Job<{ userId: string }>): Promise<void> {
    switch (job.name) {
      case 'welcome':
        await this.sendWelcomeEmail(job.data.userId);
        break;
      case 'bulk':
        await this.sendBulkEmail(job.data.userId);
        break;
    }
  }

  @OnWorkerEvent('completed')
  onCompleted(job: Job) {
    this.logger.log(`Job ${job.id} completed`);
  }

  @OnWorkerEvent('failed')
  onFailed(job: Job, error: Error) {
    this.logger.error(`Job ${job.id} failed: ${error.message}`);
  }

  private async sendWelcomeEmail(userId: string) {
    // 实际发送逻辑
  }

  private async sendBulkEmail(userId: string) {
    // 批量发送逻辑
  }
}
```

---

## 缓存

### Redis 缓存集成

```bash
npm i @nestjs/cache-manager cache-manager cache-manager-ioredis-yet ioredis
```

```typescript
// app.module.ts
@Module({
  imports: [
    CacheModule.registerAsync({
      isGlobal: true,
      imports: [ConfigModule],
      inject: [ConfigService],
      useFactory: (config: ConfigService) => ({
        store: redisStore,
        host: config.get('REDIS_HOST', 'localhost'),
        port: config.get('REDIS_PORT', 6379),
        ttl: 300,  // 默认 5 分钟
      }),
    }),
  ],
})
export class AppModule {}

// 使用 CacheInterceptor（自动缓存 GET 请求）
@Controller('products')
@UseInterceptors(CacheInterceptor)
export class ProductsController {
  @Get()
  @CacheTTL(600) // 覆盖默认 TTL，10 分钟
  @CacheKey('all_products')
  findAll() {
    return this.productsService.findAll();
  }
}

// 手动缓存控制
@Injectable()
export class ProductsService {
  constructor(@Inject(CACHE_MANAGER) private cacheManager: Cache) {}

  async findOne(id: string): Promise<Product> {
    const cacheKey = `product:${id}`;
    const cached = await this.cacheManager.get<Product>(cacheKey);
    if (cached) return cached;

    const product = await this.productRepo.findOne({ where: { id } });
    if (product) {
      await this.cacheManager.set(cacheKey, product, 600);
    }
    return product;
  }

  async update(id: string, dto: UpdateProductDto): Promise<Product> {
    const product = await this.productRepo.save({ id, ...dto });
    await this.cacheManager.del(`product:${id}`);  // 失效缓存
    await this.cacheManager.del('all_products');
    return product;
  }
}
```

---

## 测试

### 单元测试

```typescript
import { Test, TestingModule } from '@nestjs/testing';

describe('UsersService', () => {
  let service: UsersService;
  let repo: Repository<User>;

  beforeEach(async () => {
    const module: TestingModule = await Test.createTestingModule({
      providers: [
        UsersService,
        {
          provide: getRepositoryToken(User),
          useValue: {
            findOne: jest.fn(),
            find: jest.fn(),
            save: jest.fn(),
            create: jest.fn(),
            delete: jest.fn(),
          },
        },
      ],
    }).compile();

    service = module.get<UsersService>(UsersService);
    repo = module.get<Repository<User>>(getRepositoryToken(User));
  });

  describe('findOne', () => {
    it('should return a user', async () => {
      const user = { id: '1', name: 'Alice', email: 'alice@test.com' };
      jest.spyOn(repo, 'findOne').mockResolvedValue(user as User);

      const result = await service.findOne('1');
      expect(result).toEqual(user);
      expect(repo.findOne).toHaveBeenCalledWith({ where: { id: '1' } });
    });

    it('should throw NotFoundException if user not found', async () => {
      jest.spyOn(repo, 'findOne').mockResolvedValue(null);

      await expect(service.findOne('999')).rejects.toThrow(NotFoundException);
    });
  });
});
```

### E2E 测试

```typescript
import { Test, TestingModule } from '@nestjs/testing';
import { INestApplication, ValidationPipe } from '@nestjs/common';
import * as request from 'supertest';

describe('UsersController (e2e)', () => {
  let app: INestApplication;

  beforeAll(async () => {
    const moduleFixture: TestingModule = await Test.createTestingModule({
      imports: [AppModule],
    })
      .overrideProvider(UsersService)
      .useValue({
        findAll: jest.fn().mockResolvedValue([{ id: '1', name: 'Alice' }]),
        findOne: jest.fn().mockResolvedValue({ id: '1', name: 'Alice' }),
        create: jest.fn().mockResolvedValue({ id: '2', name: 'Bob' }),
      })
      .compile();

    app = moduleFixture.createNestApplication();
    app.useGlobalPipes(new ValidationPipe({ whitelist: true, transform: true }));
    await app.init();
  });

  afterAll(async () => {
    await app.close();
  });

  describe('GET /users', () => {
    it('should return an array of users', () => {
      return request(app.getHttpServer())
        .get('/users')
        .expect(200)
        .expect((res) => {
          expect(res.body).toHaveLength(1);
          expect(res.body[0].name).toBe('Alice');
        });
    });
  });

  describe('POST /users', () => {
    it('should create a user', () => {
      return request(app.getHttpServer())
        .post('/users')
        .send({ name: 'Bob', email: 'bob@test.com', password: 'securePass1' })
        .expect(201);
    });

    it('should reject invalid input', () => {
      return request(app.getHttpServer())
        .post('/users')
        .send({ name: '' })
        .expect(400);
    });
  });
});
```

### Testing 模块高级用法

```typescript
// 自动 Mock 所有依赖
const module = await Test.createTestingModule({
  providers: [UsersService],
})
  .useMocker((token) => {
    if (typeof token === 'function') {
      return createMock(token); // 使用 @golevelup/ts-jest
    }
    return {};
  })
  .compile();

// 覆盖守卫
const module = await Test.createTestingModule({
  imports: [AppModule],
})
  .overrideGuard(JwtAuthGuard)
  .useValue({ canActivate: () => true })
  .compile();

// 覆盖拦截器
const module = await Test.createTestingModule({
  imports: [AppModule],
})
  .overrideInterceptor(CacheInterceptor)
  .useValue({ intercept: (_, next) => next.handle() })
  .compile();
```

---

## 部署

### Docker

```dockerfile
# 多阶段构建
FROM node:20-alpine AS builder
WORKDIR /app
COPY package*.json ./
RUN npm ci
COPY . .
RUN npm run build

FROM node:20-alpine AS production
WORKDIR /app
COPY package*.json ./
RUN npm ci --only=production && npm cache clean --force
COPY --from=builder /app/dist ./dist

# 非 root 用户运行
RUN addgroup -g 1001 -S nodejs && adduser -S nestjs -u 1001
USER nestjs

EXPOSE 3000
CMD ["node", "dist/main"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: nestjs-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: nestjs-api
  template:
    metadata:
      labels:
        app: nestjs-api
    spec:
      containers:
        - name: api
          image: registry.example.com/nestjs-api:latest
          ports:
            - containerPort: 3000
          env:
            - name: NODE_ENV
              value: "production"
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: db-secrets
                  key: url
          resources:
            requests:
              cpu: 100m
              memory: 256Mi
            limits:
              cpu: 500m
              memory: 512Mi
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 30
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 5
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: nestjs-api
spec:
  selector:
    app: nestjs-api
  ports:
    - port: 80
      targetPort: 3000
  type: ClusterIP
```

### 健康检查

```bash
npm i @nestjs/terminus
```

```typescript
@Controller('health')
export class HealthController {
  constructor(
    private health: HealthCheckService,
    private db: TypeOrmHealthIndicator,
    private redis: MicroserviceHealthIndicator,
    private memory: MemoryHealthIndicator,
    private disk: DiskHealthIndicator,
  ) {}

  @Get()
  check() {
    return this.health.check([
      () => this.db.pingCheck('database'),
      () => this.redis.pingCheck('redis', {
        transport: Transport.REDIS,
        options: { host: 'localhost', port: 6379 },
      }),
      () => this.memory.checkHeap('memory_heap', 200 * 1024 * 1024), // 200MB
      () => this.disk.checkStorage('disk', { thresholdPercent: 0.9, path: '/' }),
    ]);
  }
}
```

### Monorepo 结构

```bash
nest generate app orders-service    # 添加子应用
nest generate lib shared            # 添加共享库
```

```
project-root/
├── apps/
│   ├── api-gateway/          # HTTP 入口
│   │   └── src/
│   ├── orders-service/       # 订单微服务
│   │   └── src/
│   └── notifications-service/ # 通知微服务
│       └── src/
├── libs/
│   ├── shared/               # 共享 DTO / 接口
│   │   └── src/
│   └── database/             # 共享数据库模块
│       └── src/
├── nest-cli.json
└── tsconfig.json
```

### Serverless 部署

```typescript
// serverless.ts (AWS Lambda)
import { NestFactory } from '@nestjs/core';
import { ExpressAdapter } from '@nestjs/platform-express';
import serverlessExpress from '@codegenie/serverless-express';
import express from 'express';

let cachedServer: any;

async function bootstrap() {
  if (cachedServer) return cachedServer;
  const expressApp = express();
  const app = await NestFactory.create(AppModule, new ExpressAdapter(expressApp));
  app.enableCors();
  await app.init();
  cachedServer = serverlessExpress({ app: expressApp });
  return cachedServer;
}

export const handler = async (event: any, context: any, callback: any) => {
  const server = await bootstrap();
  return server(event, context, callback);
};
```

---

## 性能优化

### 1. Fastify 适配器

```bash
npm i @nestjs/platform-fastify
```

```typescript
import { NestFactory } from '@nestjs/core';
import { FastifyAdapter, NestFastifyApplication } from '@nestjs/platform-fastify';

async function bootstrap() {
  const app = await NestFactory.create<NestFastifyApplication>(
    AppModule,
    new FastifyAdapter({ logger: true }),
  );
  // 注意：Fastify 需要 listen 绑定 0.0.0.0（Docker/K8s 中必需）
  await app.listen(3000, '0.0.0.0');
}
```

> **基准参考**: Fastify 适配器在简单 JSON 返回场景下吞吐量约为 Express 的 2x。实际业务中差距缩小，但在高并发场景仍有明显优势。

### 2. 压缩

```bash
npm i @nestjs/platform-express compression
```

```typescript
import compression from 'compression';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  app.use(compression({
    threshold: 1024,  // 仅压缩大于 1KB 的响应
    level: 6,
  }));
  await app.listen(3000);
}
```

### 3. 懒加载模块

```typescript
import { LazyModuleLoader } from '@nestjs/core';

@Injectable()
export class ReportService {
  constructor(private lazyModuleLoader: LazyModuleLoader) {}

  async generateReport() {
    // 仅在需要时加载重量级模块
    const { ReportModule } = await import('./report.module');
    const moduleRef = await this.lazyModuleLoader.load(() => ReportModule);
    const reportGenerator = moduleRef.get(ReportGenerator);
    return reportGenerator.generate();
  }
}
```

### 4. 集群模式

```typescript
// main.ts
import cluster from 'node:cluster';
import os from 'node:os';

async function bootstrap() {
  const app = await NestFactory.create(AppModule);
  await app.listen(3000);
}

if (cluster.isPrimary) {
  const numCPUs = os.cpus().length;
  console.log(`Master process ${process.pid}, forking ${numCPUs} workers`);
  for (let i = 0; i < numCPUs; i++) {
    cluster.fork();
  }
  cluster.on('exit', (worker) => {
    console.log(`Worker ${worker.process.pid} died, restarting...`);
    cluster.fork();
  });
} else {
  bootstrap();
}
```

> **生产建议**: 优先使用 PM2 或 K8s 水平扩展管理多进程，而非手动 cluster。手动 cluster 适合简单部署场景。

### 5. 其他优化措施

```typescript
// 启用 shutdown hooks（优雅关闭）
app.enableShutdownHooks();

// 设置全局前缀
app.setGlobalPrefix('api');

// CORS
app.enableCors({
  origin: ['https://example.com'],
  methods: ['GET', 'POST', 'PUT', 'DELETE'],
  credentials: true,
});

// Helmet 安全头
import helmet from 'helmet';
app.use(helmet());

// 速率限制
import { ThrottlerModule, ThrottlerGuard } from '@nestjs/throttler';

@Module({
  imports: [
    ThrottlerModule.forRoot([{
      ttl: 60000,   // 1 分钟
      limit: 100,   // 最多 100 次请求
    }]),
  ],
  providers: [{ provide: APP_GUARD, useClass: ThrottlerGuard }],
})
export class AppModule {}
```

---

## 常见陷阱

### 1. 循环依赖导致 undefined

```typescript
// ❌ 错误：A 和 B 互相注入，其中一个会是 undefined
@Injectable()
export class ServiceA {
  constructor(private serviceB: ServiceB) {} // 可能是 undefined
}

// ✅ 解决：使用 forwardRef 或提取公共逻辑
@Injectable()
export class ServiceA {
  constructor(
    @Inject(forwardRef(() => ServiceB)) private serviceB: ServiceB,
  ) {}
}
```

### 2. 请求作用域的性能问题

```typescript
// ❌ 不当：对所有服务使用请求作用域
@Injectable({ scope: Scope.REQUEST })
export class HeavyService {} // 每次请求都创建新实例

// ✅ 正确：仅在确实需要请求上下文的服务上使用
@Injectable()
export class HeavyService {
  doWork(requestContext: RequestContext) {
    // 通过参数传递请求上下文，保持单例
  }
}
```

### 3. 忘记导出 Provider

```typescript
// ❌ 错误：其他模块无法注入 UsersService
@Module({
  providers: [UsersService],
  // 缺少 exports
})
export class UsersModule {}

// ✅ 正确
@Module({
  providers: [UsersService],
  exports: [UsersService],
})
export class UsersModule {}
```

### 4. 异步初始化陷阱

```typescript
// ❌ 错误：同步 forRoot 中使用异步操作
@Module({})
export class DatabaseModule {
  static forRoot() {
    return {
      module: DatabaseModule,
      providers: [{
        provide: 'DB',
        useValue: connectToDb(), // Promise 未 await！
      }],
    };
  }
}

// ✅ 正确：使用 useFactory + async
@Module({})
export class DatabaseModule {
  static forRoot(): DynamicModule {
    return {
      module: DatabaseModule,
      providers: [{
        provide: 'DB',
        useFactory: async () => {
          const conn = await connectToDb();
          return conn;
        },
      }],
    };
  }
}
```

### 5. 测试中未正确清理

```typescript
// ❌ 错误：测试间共享状态导致随机失败
describe('Service', () => {
  let app: INestApplication;
  beforeAll(async () => { app = /* ... */ });
  // 测试完没有关闭

  // ✅ 正确：每次清理
  afterAll(async () => {
    await app.close();
  });
});
```

### 6. ValidationPipe 未配置 whitelist

```typescript
// ❌ 危险：客户端可注入任意属性
app.useGlobalPipes(new ValidationPipe());
// POST { name: "Alice", isAdmin: true } -> isAdmin 会被传入 DTO

// ✅ 安全：只保留 DTO 中定义的属性
app.useGlobalPipes(new ValidationPipe({
  whitelist: true,
  forbidNonWhitelisted: true,
}));
```

### 7. 生产环境 synchronize: true

```typescript
// ❌ 灾难性：自动同步会删除数据列
TypeOrmModule.forRoot({
  synchronize: true, // 绝对不能在生产环境开启
});

// ✅ 正确：使用迁移管理数据库变更
TypeOrmModule.forRoot({
  synchronize: false,
  migrations: ['dist/migrations/*.js'],
  migrationsRun: true,
});
```

---

## 最佳实践

1. **模块边界清晰**: 每个功能域一个模块，通过 exports 控制可见性
2. **DTO 验证一切输入**: 使用 class-validator + ValidationPipe(whitelist: true)
3. **依赖注入优于直接导入**: 提高可测试性，便于替换实现
4. **使用 ConfigModule 管理配置**: 不要直接读取 process.env
5. **全局异常过滤器**: 统一错误响应格式
6. **全局拦截器**: 响应包装、日志、超时控制
7. **接口而非实现**: 使用自定义 token 注入，便于切换实现
8. **健康检查端点**: 必须暴露 /health 用于 K8s 探针
9. **优雅关闭**: 启用 shutdownHooks，处理 SIGTERM
10. **日志结构化**: 使用 Pino 或 Winston，输出 JSON 格式日志

---

## 参考资料

- [NestJS 官方文档](https://docs.nestjs.com/)
- [NestJS GitHub](https://github.com/nestjs/nest)
- [Awesome NestJS](https://github.com/nestjsx/awesome-nestjs)
- [NestJS 官方示例集合](https://github.com/nestjs/nest/tree/master/sample)

---

## Agent Checklist

- [ ] 项目使用 `nest new` 或手动创建时，确认 `reflect-metadata` 和 `rxjs` 已安装
- [ ] 模块结构遵循功能域划分，每个域独立模块 + 控制器 + 服务
- [ ] 全局 ValidationPipe 配置 whitelist + forbidNonWhitelisted + transform
- [ ] 全局异常过滤器统一错误响应格式
- [ ] 数据库 synchronize 在生产环境为 false，使用迁移管理
- [ ] TypeORM 事务使用 QueryRunner 或 DataSource.transaction()
- [ ] Prisma 使用 $transaction 保证原子性
- [ ] JWT 认证使用 Passport + @nestjs/jwt，token 有过期时间
- [ ] RBAC 守卫与 @Roles() 装饰器配合使用
- [ ] WebSocket Gateway 配置认证中间件
- [ ] 微服务客户端使用 ClientProxy 通信，区分 send（请求-响应）和 emit（事件）
- [ ] GraphQL 使用 DataLoader 解决 N+1 查询问题
- [ ] 队列任务配置重试策略和失败保留策略
- [ ] 缓存使用 Redis，写操作后主动失效相关缓存
- [ ] Docker 使用多阶段构建，非 root 用户运行
- [ ] K8s 配置 liveness/readiness 探针指向 /health
- [ ] 启用 shutdownHooks 实现优雅关闭
- [ ] 启用 Helmet + CORS + ThrottlerGuard 安全措施
- [ ] 单元测试使用 Testing 模块 Mock 依赖
- [ ] E2E 测试使用 overrideProvider/overrideGuard 隔离外部依赖
- [ ] 无循环依赖；若无法避免，使用 forwardRef 并注释原因
- [ ] 生产环境使用 Fastify 适配器或 PM2/K8s 水平扩展
- [ ] ConfigModule 集中管理所有环境变量，不直接使用 process.env

---

**文档版本**: v1.0
**最后更新**: 2026-03-28
**质量评分**: 92/100
